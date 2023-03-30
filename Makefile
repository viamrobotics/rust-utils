BUF_BIN="`pwd`/bin"
PATH_WITH_TOOLS="${BUF_BIN}:${PATH}"

all: build build-example
build:
	cargo build
build-example:
	cd examples/ && cargo build
buf-clean:
	find src/gen -type f \( -iname "*.rs" ! -iname "mod.rs" \) -delete
buf-install:
	./etc/install_buf.sh $(BUF_BIN)
buf:	buf-install buf-clean
	PATH=${PATH_WITH_TOOLS} buf generate buf.build/viamrobotics/goutils --template buf.gen.yaml
	PATH=${PATH_WITH_TOOLS} buf generate buf.build/googleapis/googleapis --template buf.gen.yaml --path google/rpc --path google/api
tests/goutils:
	git clone --depth=1 https://github.com/viamrobotics/goutils.git tests/goutils
test: buf tests/goutils build
	./etc/test.sh
