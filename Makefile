clean:
	rm -rf dist/Logs dist/app Logs # Keeps .env
	cargo clean

build:
	rm -rf dist/Logs dist/app Logs # Keeps .env
	mkdir -p dist
	cargo build --release
	cp target/release/posts-notifs dist/app
	cp -n .env dist/.env

test:
	cargo test

lint:
	cargo clippy --fix

force-lint:
	cargo clippy --fix --allow-dirty --allow-staged

dev:
	RUST_SPANTRACE=1 RUST_BACKTRACE=full RUST_LIB_BACKTRACE=1 cargo run

prod: build
	cd dist && ./app

.PHONY: test lint force-lint dev prod