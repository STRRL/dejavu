//go:build darwin

package macbridge

/*
#cgo darwin CFLAGS: -mmacosx-version-min=10.15
#cgo darwin LDFLAGS: -mmacosx-version-min=10.15 -framework Cocoa -framework ApplicationServices -framework Vision -framework ImageIO
#include <stdlib.h>

int dj_screen_access(void);
int dj_request_screen_access(void);
int dj_accessibility_access(void);
int dj_request_accessibility_access(void);
char *dj_window_context(char **error_out);
char *dj_recognize_image(const char *path, char **error_out);
void dj_free_string(char *value);
*/
import "C"

import (
	"encoding/json"
	"errors"
	"unsafe"

	"github.com/strrl/dejavu/internal/model"
)

func ScreenPermission() model.PermissionState {
	if C.dj_screen_access() == 1 {
		return model.PermissionGranted
	}
	return model.PermissionRequired
}

func RequestScreenPermission() model.PermissionState {
	if C.dj_request_screen_access() == 1 {
		return model.PermissionGranted
	}
	return model.PermissionRequired
}

func AccessibilityPermission() model.PermissionState {
	if C.dj_accessibility_access() == 1 {
		return model.PermissionGranted
	}
	return model.PermissionRequired
}

func RequestAccessibilityPermission() model.PermissionState {
	if C.dj_request_accessibility_access() == 1 {
		return model.PermissionGranted
	}
	return model.PermissionRequired
}

func WindowContext() (model.WindowContext, error) {
	var errorValue *C.char
	value := C.dj_window_context(&errorValue)
	if errorValue != nil {
		defer C.dj_free_string(errorValue)
		return model.WindowContext{}, errors.New(C.GoString(errorValue))
	}
	if value == nil {
		return model.WindowContext{}, errors.New("window context is empty")
	}
	defer C.dj_free_string(value)

	var context model.WindowContext
	if err := json.Unmarshal([]byte(C.GoString(value)), &context); err != nil {
		return model.WindowContext{}, err
	}
	return context, nil
}

func Recognize(path string) ([]model.TextBlock, error) {
	pathValue := C.CString(path)
	defer C.free(unsafe.Pointer(pathValue))

	var errorValue *C.char
	value := C.dj_recognize_image(pathValue, &errorValue)
	if errorValue != nil {
		defer C.dj_free_string(errorValue)
		return nil, errors.New(C.GoString(errorValue))
	}
	if value == nil {
		return nil, errors.New("OCR result is empty")
	}
	defer C.dj_free_string(value)

	var blocks []model.TextBlock
	if err := json.Unmarshal([]byte(C.GoString(value)), &blocks); err != nil {
		return nil, err
	}
	return blocks, nil
}
