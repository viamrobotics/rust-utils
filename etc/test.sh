#!/bin/bash

SERVER_PORT=8080

if lsof -i ":$SERVER_PORT" | grep -q LISTEN; then
    echo "port :$SERVER_PORT is in use"
    exit 1
fi

pushd tests/server && ./entrypoint.sh "$SERVER_PORT" &
popd

while ! lsof -i ":$SERVER_PORT" | grep -q LISTEN; do
  sleep 0.1
done

SERVER_PORT=$SERVER_PORT cargo test --test "*"
result=$?

kill "$(lsof -t -i:$SERVER_PORT)"

exit $result
