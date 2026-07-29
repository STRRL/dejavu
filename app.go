package main

import (
	"context"
	"fmt"
	"os"
	"path/filepath"

	"github.com/strrl/dejavu/internal/capture"
	"github.com/strrl/dejavu/internal/macbridge"
	"github.com/strrl/dejavu/internal/model"
	"github.com/strrl/dejavu/internal/store"
	"github.com/wailsapp/wails/v2/pkg/runtime"
)

type App struct {
	ctx     context.Context
	store   *store.Store
	capture *capture.Service
}

func NewApp() (*App, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return nil, fmt.Errorf("find home directory: %w", err)
	}
	storeValue, err := store.Open(filepath.Join(home, ".dejavu"))
	if err != nil {
		return nil, err
	}
	return &App{
		store:   storeValue,
		capture: capture.New(storeValue),
	}, nil
}

func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
	a.capture.SetStatusCallback(func(status model.AppStatus) {
		runtime.EventsEmit(ctx, "status", status)
	})
	a.capture.Start(ctx)
}

func (a *App) shutdown(context.Context) {
	a.capture.Stop()
	_ = a.store.Close()
}

func (a *App) GetStatus() model.AppStatus {
	return a.capture.Status()
}

func (a *App) RefreshStatus() model.AppStatus {
	return a.capture.RefreshPermissions()
}

func (a *App) RequestScreenPermission() model.AppStatus {
	macbridge.RequestScreenPermission()
	return a.capture.RefreshPermissions()
}

func (a *App) RequestAccessibilityPermission() model.AppStatus {
	macbridge.RequestAccessibilityPermission()
	return a.capture.RefreshPermissions()
}

func (a *App) PauseCapture() model.AppStatus {
	a.capture.Stop()
	return a.capture.Status()
}

func (a *App) ResumeCapture() model.AppStatus {
	a.capture.Start(a.ctx)
	return a.capture.Status()
}

func (a *App) CaptureNow() {
	a.capture.RequestCapture()
}

func (a *App) Search(request model.SearchRequest) (model.SearchPage, error) {
	return a.store.Search(a.ctx, request)
}

func (a *App) GetCapture(id int64) (model.CaptureDetail, error) {
	return a.store.GetCapture(a.ctx, id)
}

func (a *App) GetStorageStats() (model.StorageStats, error) {
	return a.store.StorageStats(a.ctx)
}
