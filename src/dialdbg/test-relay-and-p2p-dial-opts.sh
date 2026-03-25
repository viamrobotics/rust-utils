#!/usr/bin/env bash
set -euo pipefail

cargo build --features dialdbg --bin viam-dialdbg 2>&1 | tail -1
BINARY=./target/debug/viam-dialdbg

HOST="<machine-fqdn>"
ENTITY="<api-key-id>"
APIKEY="<api-key>"
# TURN_URI should be the URI of a TURN server returned by the signaling server.
# Example: "turn:turn.viam.com:443"
TURN_URI="<turn-uri>"
COMMON=(-u "$HOST" -e "$ENTITY" -t api-key -c "$APIKEY" --nogrpc --nortt)

PASS=0
FAIL=0

# Extract the local ICE candidate type from dialdbg output.
# Stats print "local ICE candidate:" followed by fields including "candidate type: <type>".
# We grab the type value on the line immediately after the local candidate header.
local_candidate_type() {
  awk '/\tlocal ICE candidate:/{found=1} found && /candidate type:/{print $NF; found=0}' <<< "$1" | head -1
}

assert() {
  local name="$1" output="$2" want_type="$3"  # want_type: "relay", "!relay", or "none"
  printf '\n=== %s ===\n' "$name"
  printf '%s\n' "$output"

  local actual
  actual=$(local_candidate_type "$output")

  if [[ "$want_type" == "none" ]]; then
    # Expect connection failure — no candidates should be nominated.
    if [[ -z "$actual" ]]; then
      printf 'PASS: no candidates nominated (expected failure)\n'
      ((++PASS))
    else
      printf 'FAIL: expected no candidates, got local candidate type: %s\n' "$actual"
      ((++FAIL))
    fi
  elif [[ "$want_type" == "!relay" ]]; then
    # Expect a successful non-relay connection.
    if [[ -n "$actual" && "$actual" != "relay" ]]; then
      printf 'PASS: local candidate type: %s (not relay)\n' "$actual"
      ((++PASS))
    elif [[ -z "$actual" ]]; then
      printf 'FAIL: connection failed (no candidates nominated)\n'
      ((++FAIL))
    else
      printf 'FAIL: expected non-relay candidate, got: %s\n' "$actual"
      ((++FAIL))
    fi
  else
    # Expect a specific candidate type (e.g. "relay").
    if [[ "$actual" == "$want_type" ]]; then
      printf 'PASS: local candidate type: %s\n' "$actual"
      ((++PASS))
    elif [[ -z "$actual" ]]; then
      printf 'FAIL: connection failed (no candidates nominated)\n'
      ((++FAIL))
    else
      printf 'FAIL: expected candidate type %s, got: %s\n' "$want_type" "$actual"
      ((++FAIL))
    fi
  fi
}

run() {
  "$BINARY" "${COMMON[@]}" "$@" 2>/dev/null || true
}

# 1. Baseline — expect non-relay (host or srflx)
assert "baseline (no flags)" "$(run)" "!relay"

# 2. ForceRelay — expect relay
assert "ForceRelay" "$(run --force-relay)" "relay"

# 3. ForceRelay + TurnUri matching a valid TURN server — expect relay
assert "ForceRelay + TurnUri" "$(run --force-relay --turn-uri "$TURN_URI")" "relay"

# 4. ForceP2P — expect non-relay (host or srflx)
assert "ForceP2P" "$(run --force-p2p)" "!relay"

# 5. TurnUri alone — filters TURN options but ICE still picks host/srflx; expect non-relay
assert "TurnUri (no ForceRelay)" "$(run --turn-uri "$TURN_URI")" "!relay"

# 6. ForceRelay + non-matching TurnUri — all TURN filtered out, no relay candidates, expect failure
assert "ForceRelay + TurnUri=notexist (expect: fail)" "$(run --force-relay --turn-uri "turn:notexist.example.com:3478")" "none"

printf '\n%d passed, %d failed.\n' "$PASS" "$FAIL"
[[ $FAIL -eq 0 ]]
