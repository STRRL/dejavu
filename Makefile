.PHONY: all
all: dejavu

.PHONY: dejavu
dejavu: frontend-embbed
	cargo build --release

.PHONY: frontend-embbed
frontend-embbed: webui-export

.PHONY: webui-export
webui-export:
	cd webui && pnpm install && cp next.config.js.export next.config.js && npx next build && cp next.config.js.dev next.config.js

.PHONY: clean
clean:
	cargo clean
