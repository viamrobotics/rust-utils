# Viam Rust Utils
Utilities built in rust with use in various Viam SDKs and beyond

## Prerequisites

### Installing Rust for Mac and Unix-Like
Prior completing these steps make sure no other installations of Rust are present, for example for Mac you want to run `brew uninstall rust`

Next run the following command `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` in your favorite terminal, if you have installed rustup in the past you can just run `rustup update`. The PATH environment variable should be updated by the installer, running `which rustc` you should see something like `~/.cargo/bin/rustc` if not you may need to reload your terminal and if it still doesn't work then you should add `~/.cargo/bin` in your PATH environment variable (more info [here](https://www.rust-lang.org/tools/install))


## Repository Layout
- `src/gen` All the google and viam api proto files
- `src/dial` The implementation of gRPC over webRTC used for p2p connection to robots
- `src/ffi` FFI wrapper for dial logic
- `src/proxy` Logic for creating a unix socket to serve as connectiong proxy
- `examples` A list of examples

## Getting Started
The logic in this library is meant for use with Viam's SDKs - Rust and otherwise - rather than as a standalone product. To learn more about using the logic contained here, see the [rust-sdk](https://www.github.com/viamrobotics/viam-rust-sdk) or [python-sdk](https://www.github.com/viamrobotics/viam-python-sdk). 
If you would like to verify that this code works, you can run one of the examples by navigation to the examples folder and run `cargo run --bin test-echo` (you will need to provide your own robot's credentials to do so, or see instructions below)

### Echo Streaming Example
The echo example communicates with the goutils sample server. It demonstrates individual, streamed, and bidirectional communication. To test, navigate to your goutils clone and run

``` shell
go run rpc/examples/echo/server/cmd/main.go
```
Take note of the signaling port and replace the port value in examples/src/echo/main.rs with yours like this :

``` rust
let c = dial::DialOptions::builder()
    .uri("localhost:<your-port>")
    .without_credentials()
    .allow_downgrade()
    .connect
    .await?;
```
Then, from the `examples/` directory, run 

``` shell
cargo run --bin test-echo
```

## Using FFI (Foreign Functions Interface)
The rust sdk exposes a few functions conforming to the platform's C calling convention. Most languages support calling C code but their particular implementation is beyond the scope of this README. However we provide example in C++.

### Set up
For now we only support Unix-like systems. Before continuing make sure that GRPCCpp has been installed on your system.
Navigate to :

``` shell
cd examples/src/ffi/cpp
# Then
make buf
```

### Echo example
The echo example communicate with the goutils sample server, navigate to your goutils clone and run

``` shell
go run rpc/examples/echo/server/cmd/main.go
```

Then run 

``` shell
make ffi_echo && ./ffi_echo
```

### Robot example
The robot example communicate with a rdk server
Update the dial function with your address and secret in the file ffi_robot.cc

``` c++
dial("<robot-address>",
            "<robot-secret>",
            false, ptr);
```
Then run 

``` shell
make ffi_robot && ./ffi_robot
```

## Two Notes on Connectivity and webRTC Functionality
First: the rust SDK attempts to dial over webRTC by default. You can override this by calling `disable_webrtc()` on the dial builder, like so:

``` rust
let c = dial::DialOptions::builder()
        .uri("test-main.33vvxnbbw9.local.viam.cloud:8080") // Robot address
        .with_credentials(creds) // credentials
        .disable_webrtc() // forces gRPC connection
        .connect()
        .await?; // if the connection complete you will have a channel otherwise an error
```

Second: the rust webRTC implementation is still new, and liable to have bugs. At a minimum, we expect that calls to `ShellService::shell()` have a high likelihood of strange behavior. If you encounter any issues with streaming requests over webRTC, direct dial (by disabling webrtc as above) should resolve them. And please file a bug report! We will endeavor to be as responsive as possible, and resolve issues as quickly as possible.

## License 
Copyright 2021-2022 Viam Inc.

Apache 2.0 - See [LICENSE](https://github.com/viamrobotics/rust-utils/blob/main/LICENSE) file
