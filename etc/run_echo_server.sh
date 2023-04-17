#!/bin/bash

cd tests/goutils/rpc/examples/echo || exit 1

go run server/cmd/main.go -instance-name="localhost:$1" "$1"
