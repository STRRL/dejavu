package model

type PermissionState string

const (
	PermissionGranted     PermissionState = "granted"
	PermissionRequired    PermissionState = "required"
	PermissionUnknown     PermissionState = "unknown"
	PermissionUnsupported PermissionState = "unsupported"
)

type WindowContext struct {
	AppName              string  `json:"appName"`
	BundleID             string  `json:"bundleId"`
	ProcessID            int     `json:"processId"`
	WindowTitle          string  `json:"windowTitle"`
	WindowX              float64 `json:"windowX"`
	WindowY              float64 `json:"windowY"`
	WindowWidth          float64 `json:"windowWidth"`
	WindowHeight         float64 `json:"windowHeight"`
	AccessibilityGranted bool    `json:"accessibilityGranted"`
}

type TextBlock struct {
	ID         int64   `json:"id"`
	CaptureID  int64   `json:"captureId"`
	Text       string  `json:"text"`
	Confidence float64 `json:"confidence"`
	X          float64 `json:"x"`
	Y          float64 `json:"y"`
	Width      float64 `json:"width"`
	Height     float64 `json:"height"`
}

type CaptureInput struct {
	DisplayIndex   int
	Width          int
	Height         int
	ImageFile      string
	ThumbnailFile  string
	ImageBytes     int64
	ThumbnailBytes int64
	ChangeRatio    float64
	OCRStatus      string
	OCRError       string
	Blocks         []TextBlock
}

type MomentInput struct {
	CapturedAt int64
	Window     WindowContext
	Captures   []CaptureInput
}

type HighlightRange struct {
	Start int `json:"start"`
	End   int `json:"end"`
}

type SearchRequest struct {
	Query  string `json:"query"`
	Offset int    `json:"offset"`
	Limit  int    `json:"limit"`
}

type SearchResult struct {
	ID            int64            `json:"id"`
	CapturedAt    int64            `json:"capturedAt"`
	DisplayIndex  int              `json:"displayIndex"`
	Width         int              `json:"width"`
	Height        int              `json:"height"`
	AppName       string           `json:"appName"`
	BundleID      string           `json:"bundleId"`
	WindowTitle   string           `json:"windowTitle"`
	Snippet       string           `json:"snippet"`
	Highlights    []HighlightRange `json:"highlights"`
	MatchingBoxes []TextBlock      `json:"matchingBoxes"`
	OCRStatus     string           `json:"ocrStatus"`
}

type SearchPage struct {
	Results    []SearchResult `json:"results"`
	NextOffset int            `json:"nextOffset"`
	HasMore    bool           `json:"hasMore"`
}

type CaptureDetail struct {
	ID           int64       `json:"id"`
	CapturedAt   int64       `json:"capturedAt"`
	DisplayIndex int         `json:"displayIndex"`
	Width        int         `json:"width"`
	Height       int         `json:"height"`
	AppName      string      `json:"appName"`
	BundleID     string      `json:"bundleId"`
	ProcessID    int         `json:"processId"`
	WindowTitle  string      `json:"windowTitle"`
	WindowX      float64     `json:"windowX"`
	WindowY      float64     `json:"windowY"`
	WindowWidth  float64     `json:"windowWidth"`
	WindowHeight float64     `json:"windowHeight"`
	OCRStatus    string      `json:"ocrStatus"`
	OCRError     string      `json:"ocrError"`
	Blocks       []TextBlock `json:"blocks"`
}

type AppStatus struct {
	State                   string          `json:"state"`
	ScreenPermission        PermissionState `json:"screenPermission"`
	AccessibilityPermission PermissionState `json:"accessibilityPermission"`
	LastCapturedAt          int64           `json:"lastCapturedAt"`
	CaptureCount            int64           `json:"captureCount"`
	LastError               string          `json:"lastError"`
}

type DailyStorage struct {
	Date       string `json:"date"`
	ImageBytes int64  `json:"imageBytes"`
	Count      int64  `json:"count"`
}

type StorageStats struct {
	ImageBytes    int64          `json:"imageBytes"`
	DatabaseBytes int64          `json:"databaseBytes"`
	CaptureCount  int64          `json:"captureCount"`
	Daily         []DailyStorage `json:"daily"`
}
