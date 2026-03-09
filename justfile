mod src-tauri

fix: src-tauri::fix

fmt: src-tauri::fmt

lint: src-tauri::lint

dev:
    cargo tauri dev

build:
    cargo tauri build

build-debug:
    cargo tauri build --config src-tauri/tauri.debug.conf.json

clean: src-tauri::clean