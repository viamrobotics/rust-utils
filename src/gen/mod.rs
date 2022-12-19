pub mod proto {
    pub mod rpc {
        pub mod webrtc {
            pub mod v1 {
                include!("proto.rpc.webrtc.v1.rs");
            }
        }
        pub mod examples {
            pub mod echo {
                pub mod v1 {
                    include!("proto.rpc.examples.echo.v1.rs");
                }
            }
            pub mod echoresource {
                pub mod v1 {
                    include!("proto.rpc.examples.echoresource.v1.rs");
                }
            }
        }
        pub mod v1 {
            include!("proto.rpc.v1.rs");
        }
    }
}
pub mod google {
    pub mod rpc {
        include!("google.rpc.rs");
    }
    pub mod api {
        include!("google.api.rs");
    }
}
