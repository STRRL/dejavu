# Dejavu

Dejavu is a private visual memory app for macOS. It saves changed screens, reads visible text with Apple Vision, and makes past moments searchable.

## Current features

Dejavu captures every active display every five seconds. Similar images are skipped. The active application and focused window are saved when Accessibility access is available.

Search uses SQLite FTS5. Latin text uses the unicode61 tokenizer. Chinese, Japanese, and Korean text uses the trigram tokenizer. Very short CJK searches use a plain text fallback.

All data stays under `~/.dejavu/`. The app shows image use by day, total image use, and database use. It does not delete old data.

## Development

Install Wails v2, Go, Node.js, and Xcode command line tools.

Run `wails dev` for development.

Run `wails build` to create the macOS application.

## Permissions

Screen Recording access is required for capture. Accessibility access is optional and adds the active application and focused window details.

## Future capture work

The first release uses a small grayscale image to skip near duplicates. A future version should use ScreenCaptureKit frame events and changed regions, then use an adaptive image similarity check when needed.
