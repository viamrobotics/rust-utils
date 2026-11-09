#!/bin/bash

# Checks that no Mach-O object in the given files targets a macOS version newer
# than the deployment target pinned in .cargo/config.toml. Objects built for a
# newer macOS than the one a consumer links against make the linker warn and can
# leave that consumer with binaries it cannot run.

if [[ -z "$1" ]]; then
    echo "usage: $0 <file>..."
    exit 1
fi

CONFIG="$(dirname "$0")/../.cargo/config.toml"
EXPECTED=$(sed -n 's/^MACOSX_DEPLOYMENT_TARGET = "\(.*\)"$/\1/p' "$CONFIG")
if [[ -z "$EXPECTED" ]]; then
    echo "no MACOSX_DEPLOYMENT_TARGET found in $CONFIG"
    exit 1
fi

status=0
for file in "$@"; do
    versions=$(otool -l "$file" | awk '
        $1 == "cmd" { cmd = $2 }
        cmd == "LC_BUILD_VERSION" && $1 == "minos" { print $2 }
        cmd == "LC_VERSION_MIN_MACOSX" && $1 == "version" { print $2 }
    ' | sort -u -V)

    if [[ -z "$versions" ]]; then
        echo "$file: no macOS version information found"
        status=1
        continue
    fi

    echo "$file: minimum macOS versions: $(echo "$versions" | tr '\n' ' ')"
    for version in $versions; do
        # the newest of the two versions is the expected one unless $version is newer
        if [[ "$(printf '%s\n%s\n' "$EXPECTED" "$version" | sort -V | tail -n1)" != "$EXPECTED" ]]; then
            echo "$file: built for macOS $version, newer than the pinned $EXPECTED"
            status=1
        fi
    done
done

exit $status
