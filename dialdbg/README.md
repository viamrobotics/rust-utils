# Viam Dial Debugger
A CLI tool to give information on how Viam rust utils' dial function makes connections.

## Installation
TODO(RSDK-3446): ensure installation constructions are correct.

viam-dialdbg can be installed via homebrew.

``` shell
brew install viam-dialdbg
```

## Usage examples

``` shell
viam-dialdbg --uri myremote.com --credential mycredential
```
Prints debug information to STDOUT for connecting from this machine to "myremote.com" using "mycredential" as a credential. Prints information on WebRTC connection establishment, gRPC connection establishment, and average round-trip-times for both. Prints discovered mDNS addresses on the subnet if mDNS could not be used to connect.

``` shell
viam-dialdbg --uri myremote.com --credential mycredential --output ./foo.txt
```
Same as above, but outputs debug information to ./foo.txt (./foo.txt will be overwritten).

``` shell
viam-dialdbg --uri myremote.com --credential mycredential --output ./foo.txt --credential-type bar
```
Same as above, but uses "bar" credential type for "mycredential".

``` shell
viam-dialdbg --uri myremote.com --credential mycredential --output ./foo.txt --credential-type bar --nogrpc
```
Same as above, but only examines WebRTC connection establishment.

Use `viam-dialdbg --help` for more information.

## License
Copyright 2023 Viam Inc.

Apache 2.0 - See [LICENSE](https://github.com/viamrobotics/rust-utils/blob/main/dialdbg/LICENSE) file
