#!/bin/bash

if lsof -i :8080 | grep -q LISTEN; then
    echo "port :8080 is in use"
    exit 1
fi

pushd tests/server && ./entrypoint.sh &
popd

while ! lsof -i :8080 | grep -q LISTEN; do
  sleep 0.1
done

cargo test --test "*"
result=$?

kill "$(lsof -t -i:8080)"

exit $result
