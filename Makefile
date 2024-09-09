clean:
	rm -rf dist/Logs dist/app Logs # Keeps .env
	cargo clean

build:
	rm -rf dist/Logs dist/app Logs # Keeps .env
	mkdir -p dist
	cargo build --release
	cp target/release/app dist/app
	cp -n .env dist/.env

lint:
	cargo clippy --all-targets --all-features --color always

lint-fix:
	cargo clippy --fix --all-targets --all-features --color always

force-lint-fix:
	cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features --color always

dev:
	RUST_SPANTRACE=1 RUST_BACKTRACE=full RUST_LIB_BACKTRACE=1 cargo run

prod: build
	cd dist && ./app

.PHONY: test lint force-lint dev prod