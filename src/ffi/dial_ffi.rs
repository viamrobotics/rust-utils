//! # Viam C API
//!
//! This module exposes a C API allowing a user to communicate with a Robot using any language able to call C functions without having
//! to implement webRTC or authentication. The module creates a UDS socket that a gRPC client can connect to
//!

use http::uri::Uri;
use std::{ptr, time::Duration};
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tracing::Level;

use crate::rpc::dial::{
    DialBuilder, DialOptions, RPCCredentials, ViamChannel, WithCredentials, WithoutCredentials,
};
use libc::c_char;

use crate::proxy;
use hyper::Server;
use std::ffi::{CStr, CString};
use tower::{make::Shared, ServiceBuilder};
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};

use anyhow::Result;

use crate::proxy::grpc_proxy::GRPCProxy;

/// The DialFfi interface, returned as a pointer by init_rust_runtime. User should keep this pointer until freeing the runtime.
pub struct DialFfi {
    runtime: Option<Runtime>,
    sigs: Option<Vec<oneshot::Sender<()>>>,
    channels: Vec<ViamChannel>,
}

impl Drop for DialFfi {
    fn drop(&mut self) {
        log::debug!("FFI runtime closing");
        if let Some(r) = self.runtime.take() {
            r.shutdown_timeout(Duration::from_secs(1));
        }
    }
}

impl DialFfi {
    fn new() -> Self {
        Self {
            runtime: Some(Runtime::new().unwrap()),
            sigs: None,
            channels: vec![],
        }
    }
    fn push_signal(&mut self, sig: oneshot::Sender<()>) {
        match self.sigs {
            Some(ref mut v) => v.push(sig),
            None => {
                let v: Vec<oneshot::Sender<()>> = vec![sig];
                self.sigs = Some(v);
            }
        }
    }
}
/// Initialize a tokio runtime to run a gRPC client/sever, user should call this function before trying to dial to a Robot
/// Returns a pointer to a [`DialFfi`]
#[no_mangle]
pub extern "C" fn init_rust_runtime() -> Box<DialFfi> {
    let _ = tracing_subscriber::fmt::try_init();
    Box::new(DialFfi::new())
}

fn dial_without_cred(
    uri: String,
    allow_insec: bool,
    disable_webrtc: bool,
) -> Result<DialBuilder<WithoutCredentials>> {
    let c = DialOptions::builder().uri(&uri).without_credentials();
    let c = if disable_webrtc {
        c.disable_webrtc()
    } else {
        c
    };
    let c = if allow_insec { c.allow_downgrade() } else { c };
    Ok(c)
}

fn dial_with_cred(
    uri: String,
    r#type: &str,
    payload: &str,
    allow_insec: bool,
    disable_webrtc: bool,
) -> Result<DialBuilder<WithCredentials>> {
    let creds = RPCCredentials::new(None, String::from(r#type), String::from(payload));
    let c = DialOptions::builder().uri(&uri).with_credentials(creds);
    let c = if disable_webrtc {
        c.disable_webrtc()
    } else {
        c
    };
    let c = if allow_insec { c.allow_downgrade() } else { c };
    Ok(c)
}
/// Returns a path to a UDS proxy to a robot
/// # Safety
///
/// This function must be called from another language. See [`dial`](mod@crate::rpc::dial) for dial from rust
/// The function returns a path to a UDS as a [`c_char`], the string should be freed with free_string when not needed anymore.
/// When falling to dial it will return a NULL pointer
/// # Arguments
/// * `c_uri` a C-style string representing the address of robot you want to connect to
/// * `c_type` a C-style string representing the type of robot's secret you want to use, set to NULL if you don't need authentication
/// * `c_payload` a C-style string that is the robot's secret, set to NULL if you don't need authentication
/// * `c_allow_insecure` a bool, set to true when allowing insecure connection to your robot
/// * `rt_ptr` a pointer to a rust runtime previously obtained with init_rust_runtime
#[no_mangle]
pub unsafe extern "C" fn dial(
    c_uri: *const c_char,
    c_type: *const c_char,
    c_payload: *const c_char,
    c_allow_insec: bool,
    rt_ptr: Option<&mut DialFfi>,
) -> *mut c_char {
    let uri = {
        if c_uri.is_null() {
            return ptr::null_mut();
        }
        let ur = match Uri::from_maybe_shared(CStr::from_ptr(c_uri).to_bytes()) {
            Ok(ur) => ur,
            Err(e) => {
                log::error!("Sorry {e:?} is not a valid URI");
                return ptr::null_mut();
            }
        };
        ur
    };
    let allow_insec = c_allow_insec;
    let ctx = match rt_ptr {
        Some(rt) => rt,
        None => {
            return ptr::null_mut();
        }
    };
    let runtime = match &ctx.runtime {
        Some(r) => r,
        None => {
            return ptr::null_mut();
        }
    };
    let conn = match runtime.block_on(async { proxy::uds::UDSConnector::new_random() }) {
        Ok(conn) => conn,
        Err(e) => {
            log::error!("Error creating the UDS proxy {e:?}");
            return ptr::null_mut();
        }
    };
    let path = match CString::new(conn.get_path()) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Error getting the path {e:?}");
            return ptr::null_mut();
        }
    };
    let (tx, rx) = oneshot::channel::<()>();
    let uri_str = uri.to_string();

    // if the uri is local then we can connect directly.
    let disable_webrtc;
    if let Some(host) = uri.host() {
        disable_webrtc = host.contains(".local") || host.contains("localhost");
    } else {
        disable_webrtc = uri_str.contains(".local") || uri_str.contains("localhost");
    }
    let r#type = {
        match c_type.is_null() {
            true => None,
            false => Some(CStr::from_ptr(c_type)),
        }
    };
    let payload = {
        match c_payload.is_null() {
            true => None,
            false => Some(CStr::from_ptr(c_payload)),
        }
    };
    let (server, channel) = match runtime.block_on(async move {
        let channel = match (r#type, payload) {
            (Some(t), Some(p)) => {
                dial_with_cred(
                    uri_str,
                    t.to_str()?,
                    p.to_str()?,
                    allow_insec,
                    disable_webrtc,
                )?
                .connect()
                .await
            }
            (None, None) => {
                let c = dial_without_cred(uri_str, allow_insec, disable_webrtc)?;
                c.connect().await
            }
            (None, Some(_)) => Err(anyhow::anyhow!("Error missing credential: type")),
            (Some(_), None) => Err(anyhow::anyhow!("Error missing credential: payload")),
        }?;
        let dial = channel.clone();
        let g = GRPCProxy::new(dial, uri);
        let service = ServiceBuilder::new()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    ),
            )
            .service(g);
        let server = Server::builder(conn)
            .http2_only(true)
            .serve(Shared::new(service));
        Ok::<_, Box<dyn std::error::Error>>((server, channel))
    }) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Error building GRPC proxy reason : {e:?}");
            return ptr::null_mut();
        }
    };
    ctx.channels.push(channel);
    let server = server.with_graceful_shutdown(async {
        rx.await.ok();
    });
    let _ = runtime.spawn(async {
        let _ = server.await;
    });
    ctx.push_signal(tx);
    path.into_raw()
}

/// This function must be used to free the path returned by the [`dial`] function
/// # Safety
///
/// The function must not be called more than once with the same pointer
/// # Arguments
/// * `c_char` a pointer to the string returned by [`dial`]
#[no_mangle]
pub unsafe extern "C" fn free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    log::debug!("freeing string: {s:?}");
    let _ = CString::from_raw(s);
}

/// This function must be used the free a rust runtime returned by [`init_rust_runtime`] the function will signal any
/// opened server to shutdown. Further transaction on any UDS will not work anymore.
/// # Safety
///
/// The function must not be called more than once with the same pointer
/// # Arguments
/// * `rt_prt` a pointer to the string returned by [`init_rust_runtime`]
#[no_mangle]
pub extern "C" fn free_rust_runtime(rt_ptr: Option<Box<DialFfi>>) -> i32 {
    let mut ctx = match rt_ptr {
        Some(ctx) => ctx,
        None => {
            return -1;
        }
    };
    if let Some(sigs) = ctx.sigs.take() {
        for sig in sigs {
            let _ = sig.send(());
        }
    }

    for channel in &ctx.channels {
        match channel {
            ViamChannel::Direct(_) => (),
            ViamChannel::DirectPreAuthorized(_) => (),
            ViamChannel::WebRTC(chan) => ctx
                .runtime
                .as_ref()
                .map(|rt| rt.block_on(async move { chan.close().await }))
                .unwrap_or_default(),
        }
    }
    log::debug!("Freeing rust runtime");
    0
}
