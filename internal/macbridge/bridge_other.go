//go:build !darwin

package macbridge

import (
	"errors"

	"github.com/strrl/dejavu/internal/model"
)

func ScreenPermission() model.PermissionState {
	return model.PermissionUnsupported
}

func RequestScreenPermission() model.PermissionState {
	return model.PermissionUnsupported
}

func AccessibilityPermission() model.PermissionState {
	return model.PermissionUnsupported
}

func RequestAccessibilityPermission() model.PermissionState {
	return model.PermissionUnsupported
}

func WindowContext() (model.WindowContext, error) {
	return model.WindowContext{}, errors.New("window context is not supported")
}

func Recognize(string) ([]model.TextBlock, error) {
	return nil, errors.New("OCR is not supported")
}
