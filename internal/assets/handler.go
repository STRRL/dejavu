package assets

import (
	"context"
	"errors"
	"net/http"
	"os"
	"strconv"
	"strings"

	"github.com/strrl/dejavu/internal/store"
)

type Handler struct {
	store *store.Store
}

func New(storeValue *store.Store) *Handler {
	return &Handler{store: storeValue}
}

func (h *Handler) Middleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(response http.ResponseWriter, request *http.Request) {
		if strings.HasPrefix(request.URL.Path, "/capture/") {
			h.ServeHTTP(response, request)
			return
		}
		next.ServeHTTP(response, request)
	})
}

func (h *Handler) ServeHTTP(response http.ResponseWriter, request *http.Request) {
	parts := strings.Split(strings.Trim(request.URL.Path, "/"), "/")
	if len(parts) != 3 || parts[0] != "capture" {
		http.NotFound(response, request)
		return
	}

	id, err := strconv.ParseInt(parts[1], 10, 64)
	if err != nil || id <= 0 {
		http.NotFound(response, request)
		return
	}
	thumbnail := parts[2] == "thumbnail"
	if parts[2] != "image" && !thumbnail {
		http.NotFound(response, request)
		return
	}

	path, err := h.store.ImagePath(context.Background(), id, thumbnail)
	if errors.Is(err, os.ErrNotExist) {
		http.NotFound(response, request)
		return
	}
	if err != nil {
		http.Error(response, "Could not load image", http.StatusInternalServerError)
		return
	}

	response.Header().Set("Cache-Control", "no-store")
	response.Header().Set("Content-Type", "image/jpeg")
	http.ServeFile(response, request, path)
}
