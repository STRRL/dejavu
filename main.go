package main

import (
	"embed"

	"github.com/strrl/dejavu/internal/assets"
	"github.com/wailsapp/wails/v2"
	"github.com/wailsapp/wails/v2/pkg/options"
	"github.com/wailsapp/wails/v2/pkg/options/assetserver"
)

//go:embed all:frontend/dist
var webAssets embed.FS

func main() {
	app, err := NewApp()
	if err != nil {
		println("Error:", err.Error())
		return
	}
	imageHandler := assets.New(app.store)

	err = wails.Run(&options.App{
		Title:     "Dejavu",
		Width:     1280,
		Height:    820,
		MinWidth:  920,
		MinHeight: 640,
		AssetServer: &assetserver.Options{
			Assets:     webAssets,
			Handler:    imageHandler,
			Middleware: imageHandler.Middleware,
		},
		BackgroundColour: &options.RGBA{R: 245, G: 242, B: 235, A: 1},
		OnStartup:        app.startup,
		OnShutdown:       app.shutdown,
		Bind: []interface{}{
			app,
		},
	})
	if err != nil {
		println("Error:", err.Error())
	}
}
