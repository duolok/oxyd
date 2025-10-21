BINARY_NAME := "oxyd"
SYSTEM_PATH := "/usr/local/bin/"

build:
    cargo build --release

install:
    just build
    sudo cp target/release/{{BINARY_NAME}} {{SYSTEM_PATH}}/

clean:
    cargo clean

uninstall:
    sudo rm -f {{SYSTEM_PATH}}/{{BINARY_NAME}}

run:
    target/release/{{BINARY_NAME}}
