BUF_BIN="`pwd`/bin"
PATH_WITH_TOOLS="${BUF_BIN}:${PATH}"

all: build build-example
build:
	cargo build
build-example:
	cd examples/ && cargo build
run-example:
	cd examples/ && RUST_LOG=debug cargo run --bin test-echo
buf-clean:
	find src/gen -type f \( -iname "*.rs" ! -iname "mod.rs" \) -delete
buf-install:
	./etc/install_buf.sh $(BUF_BIN)
buf:	buf-install buf-clean
	PATH=${PATH_WITH_TOOLS} buf generate buf.build/viamrobotics/goutils --template buf.gen.yaml
	PATH=${PATH_WITH_TOOLS} buf generate buf.build/googleapis/googleapis --template buf.gen.yaml --path google/rpc --path google/api
test: buf build
	./etc/test.sh
