mod src-tauri

default: dev

fix: src-tauri::fix

fmt: src-tauri::fmt

lint: src-tauri::lint

dev:
    cargo tauri dev

build:
    cargo tauri build

build-debug:
    cargo tauri build --config src-tauri/tauri.debug.conf.json

pre-commit: src-tauri::pre-commit

clean: src-tauri::clean