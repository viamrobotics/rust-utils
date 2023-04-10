#!/bin/bash

SERVER_PORT=8081

if lsof -i ":$SERVER_PORT" | grep -q LISTEN; then
    echo "port :$SERVER_PORT is in use"
    exit 1
fi

./etc/run_echo_server.sh "$SERVER_PORT" &

while ! lsof -i ":$SERVER_PORT" | grep -q LISTEN; do
  sleep 0.1
done

SERVER_PORT=$SERVER_PORT cargo test --test "*"
result=$?

kill "$(lsof -t -i:$SERVER_PORT)"

exit $result
