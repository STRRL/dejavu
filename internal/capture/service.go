package capture

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"image"
	"image/color"
	"image/jpeg"
	"os"
	"path/filepath"
	"sync"
	"time"

	"github.com/kbinani/screenshot"
	"github.com/strrl/dejavu/internal/macbridge"
	"github.com/strrl/dejavu/internal/model"
	"github.com/strrl/dejavu/internal/store"
)

const (
	captureInterval = 5 * time.Second
	signatureSize   = 64
	pixelThreshold  = 12
	changeThreshold = 0.005
)

type Service struct {
	store      *store.Store
	mu         sync.RWMutex
	state      model.AppStatus
	last       map[int][]uint8
	cancel     context.CancelFunc
	onChange   func(model.AppStatus)
	captureNow chan struct{}
	wait       sync.WaitGroup
}

func New(storeValue *store.Store) *Service {
	return &Service{
		store: storeValue,
		state: model.AppStatus{
			State:                   "paused",
			ScreenPermission:        macbridge.ScreenPermission(),
			AccessibilityPermission: macbridge.AccessibilityPermission(),
		},
		last:       make(map[int][]uint8),
		captureNow: make(chan struct{}, 1),
	}
}

func (s *Service) SetStatusCallback(callback func(model.AppStatus)) {
	s.mu.Lock()
	s.onChange = callback
	s.mu.Unlock()
}

func (s *Service) Start(ctx context.Context) {
	s.mu.Lock()
	if s.cancel != nil {
		s.mu.Unlock()
		return
	}
	runContext, cancel := context.WithCancel(ctx)
	s.cancel = cancel
	s.state.ScreenPermission = macbridge.ScreenPermission()
	s.state.AccessibilityPermission = macbridge.AccessibilityPermission()
	if s.state.ScreenPermission == model.PermissionGranted {
		s.state.State = "running"
	} else {
		s.state.State = "waiting"
	}
	state := s.state
	callback := s.onChange
	s.mu.Unlock()
	notify(callback, state)

	s.wait.Add(1)
	go func() {
		defer s.wait.Done()
		s.run(runContext)
	}()
}

func (s *Service) Stop() {
	s.mu.Lock()
	if s.cancel != nil {
		s.cancel()
		s.cancel = nil
	}
	s.state.State = "paused"
	state := s.state
	callback := s.onChange
	s.mu.Unlock()
	notify(callback, state)
	s.wait.Wait()
}

func (s *Service) RequestCapture() {
	select {
	case s.captureNow <- struct{}{}:
	default:
	}
}

func (s *Service) Status() model.AppStatus {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.state
}

func (s *Service) RefreshPermissions() model.AppStatus {
	s.mu.Lock()
	s.state.ScreenPermission = macbridge.ScreenPermission()
	s.state.AccessibilityPermission = macbridge.AccessibilityPermission()
	if s.cancel != nil && s.state.ScreenPermission == model.PermissionGranted {
		s.state.State = "running"
	}
	state := s.state
	callback := s.onChange
	s.mu.Unlock()
	notify(callback, state)
	if state.State == "running" {
		s.RequestCapture()
	}
	return state
}

func (s *Service) run(ctx context.Context) {
	timer := time.NewTicker(captureInterval)
	defer timer.Stop()
	s.captureOnce(ctx)

	for {
		select {
		case <-ctx.Done():
			return
		case <-timer.C:
			s.captureOnce(ctx)
		case <-s.captureNow:
			s.captureOnce(ctx)
		}
	}
}

func (s *Service) captureOnce(ctx context.Context) {
	if macbridge.ScreenPermission() != model.PermissionGranted {
		s.updateState(func(state *model.AppStatus) {
			state.State = "waiting"
			state.ScreenPermission = model.PermissionRequired
		})
		return
	}

	window, err := macbridge.WindowContext()
	if err != nil {
		window = model.WindowContext{
			AccessibilityGranted: macbridge.AccessibilityPermission() == model.PermissionGranted,
		}
	}

	now := time.Now()
	input := model.MomentInput{
		CapturedAt: now.Unix(),
		Window:     window,
	}
	written := make([]string, 0)
	signatures := make(map[int][]uint8)

	for displayIndex := 0; displayIndex < screenshot.NumActiveDisplays(); displayIndex++ {
		select {
		case <-ctx.Done():
			removeFiles(written)
			return
		default:
		}

		screen, err := screenshot.CaptureDisplay(displayIndex)
		if err != nil {
			s.fail(fmt.Errorf("capture display %d: %w", displayIndex, err))
			continue
		}

		signature := makeSignature(screen)
		ratio := changedRatio(s.last[displayIndex], signature)

		// TODO Replace polling with ScreenCaptureKit events and adaptive capture.
		if previous := s.last[displayIndex]; previous != nil && ratio < changeThreshold {
			continue
		}
		captureInput, paths, err := s.saveCapture(now, displayIndex, screen, ratio)
		if err != nil {
			s.fail(err)
			continue
		}
		signatures[displayIndex] = signature
		written = append(written, paths...)
		input.Captures = append(input.Captures, captureInput)
	}

	if len(input.Captures) == 0 {
		return
	}
	if err := s.store.SaveMoment(ctx, input); err != nil {
		removeFiles(written)
		s.fail(err)
		return
	}
	for displayIndex, signature := range signatures {
		s.last[displayIndex] = signature
	}

	count, countErr := s.store.CaptureCount(ctx)
	s.updateState(func(state *model.AppStatus) {
		state.State = "running"
		state.ScreenPermission = model.PermissionGranted
		state.AccessibilityPermission = macbridge.AccessibilityPermission()
		state.LastCapturedAt = input.CapturedAt
		state.LastError = ""
		if countErr == nil {
			state.CaptureCount = count
		}
	})
}

func (s *Service) saveCapture(
	now time.Time,
	displayIndex int,
	screen image.Image,
	changeRatio float64,
) (model.CaptureInput, []string, error) {
	datePath := now.Format("2006/01/02")
	directory := filepath.Join(s.store.ImageRoot(), datePath)
	if err := os.MkdirAll(directory, 0o700); err != nil {
		return model.CaptureInput{}, nil, fmt.Errorf("create capture directory: %w", err)
	}

	suffix, err := randomSuffix()
	if err != nil {
		return model.CaptureInput{}, nil, err
	}
	name := fmt.Sprintf("%s_d%d_%s", now.Format("150405.000"), displayIndex, suffix)
	imageRelative := filepath.Join(datePath, name+".jpg")
	thumbRelative := filepath.Join(datePath, name+"_thumb.jpg")
	imagePath := filepath.Join(s.store.ImageRoot(), imageRelative)
	thumbPath := filepath.Join(s.store.ImageRoot(), thumbRelative)

	if err := writeJPEG(imagePath, screen, 82); err != nil {
		return model.CaptureInput{}, nil, err
	}
	thumbnail := resizeToWidth(screen, 480)
	if err := writeJPEG(thumbPath, thumbnail, 75); err != nil {
		removeFiles([]string{imagePath})
		return model.CaptureInput{}, nil, err
	}

	imageInfo, err := os.Stat(imagePath)
	if err != nil {
		removeFiles([]string{imagePath, thumbPath})
		return model.CaptureInput{}, nil, fmt.Errorf("read capture size: %w", err)
	}
	thumbInfo, err := os.Stat(thumbPath)
	if err != nil {
		removeFiles([]string{imagePath, thumbPath})
		return model.CaptureInput{}, nil, fmt.Errorf("read thumbnail size: %w", err)
	}

	blocks, ocrErr := macbridge.Recognize(imagePath)
	ocrStatus := "done"
	ocrError := ""
	if ocrErr != nil {
		ocrStatus = "failed"
		ocrError = ocrErr.Error()
		blocks = []model.TextBlock{}
	}

	bounds := screen.Bounds()
	return model.CaptureInput{
		DisplayIndex:   displayIndex,
		Width:          bounds.Dx(),
		Height:         bounds.Dy(),
		ImageFile:      imageRelative,
		ThumbnailFile:  thumbRelative,
		ImageBytes:     imageInfo.Size(),
		ThumbnailBytes: thumbInfo.Size(),
		ChangeRatio:    changeRatio,
		OCRStatus:      ocrStatus,
		OCRError:       ocrError,
		Blocks:         blocks,
	}, []string{imagePath, thumbPath}, nil
}

func (s *Service) fail(err error) {
	s.updateState(func(state *model.AppStatus) {
		state.LastError = err.Error()
	})
}

func (s *Service) updateState(change func(*model.AppStatus)) {
	s.mu.Lock()
	change(&s.state)
	state := s.state
	callback := s.onChange
	s.mu.Unlock()
	notify(callback, state)
}

func notify(callback func(model.AppStatus), state model.AppStatus) {
	if callback != nil {
		callback(state)
	}
}

func randomSuffix() (string, error) {
	value := make([]byte, 4)
	if _, err := rand.Read(value); err != nil {
		return "", fmt.Errorf("create capture name: %w", err)
	}
	return hex.EncodeToString(value), nil
}

func writeJPEG(path string, value image.Image, quality int) error {
	file, err := os.OpenFile(path, os.O_CREATE|os.O_EXCL|os.O_WRONLY, 0o600)
	if err != nil {
		return fmt.Errorf("create image: %w", err)
	}
	encodeErr := jpeg.Encode(file, value, &jpeg.Options{Quality: quality})
	closeErr := file.Close()
	if encodeErr != nil {
		_ = os.Remove(path)
		return fmt.Errorf("encode image: %w", encodeErr)
	}
	if closeErr != nil {
		_ = os.Remove(path)
		return fmt.Errorf("close image: %w", closeErr)
	}
	return nil
}

func makeSignature(value image.Image) []uint8 {
	bounds := value.Bounds()
	result := make([]uint8, signatureSize*signatureSize)
	for y := 0; y < signatureSize; y++ {
		sourceY := bounds.Min.Y + y*bounds.Dy()/signatureSize
		for x := 0; x < signatureSize; x++ {
			sourceX := bounds.Min.X + x*bounds.Dx()/signatureSize
			gray := color.GrayModel.Convert(value.At(sourceX, sourceY)).(color.Gray)
			result[y*signatureSize+x] = gray.Y
		}
	}
	return result
}

func changedRatio(previous []uint8, current []uint8) float64 {
	if len(previous) != len(current) || len(current) == 0 {
		return 1
	}
	changed := 0
	for index, value := range current {
		delta := int(value) - int(previous[index])
		if delta < 0 {
			delta = -delta
		}
		if delta > pixelThreshold {
			changed++
		}
	}
	return float64(changed) / float64(len(current))
}

func resizeToWidth(source image.Image, width int) image.Image {
	bounds := source.Bounds()
	if bounds.Dx() <= width {
		return source
	}
	height := bounds.Dy() * width / bounds.Dx()
	target := image.NewRGBA(image.Rect(0, 0, width, height))
	for y := 0; y < height; y++ {
		sourceY := bounds.Min.Y + y*bounds.Dy()/height
		for x := 0; x < width; x++ {
			sourceX := bounds.Min.X + x*bounds.Dx()/width
			target.Set(x, y, source.At(sourceX, sourceY))
		}
	}
	return target
}

func removeFiles(paths []string) {
	for _, path := range paths {
		_ = os.Remove(path)
	}
}
