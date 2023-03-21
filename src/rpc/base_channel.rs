use anyhow::Result;
use std::{
    fmt::Debug,
    panic::{catch_unwind, set_hook, AssertUnwindSafe},
    sync::{
        atomic::{AtomicBool, AtomicPtr, Ordering},
        Arc,
    },
};
use webrtc::{data_channel::RTCDataChannel, peer_connection::RTCPeerConnection};

// see golang/client_stream.go
/// The base components to a webRTC channel, used on both client and server sides.
pub struct WebRTCBaseChannel {
    pub(crate) peer_connection: Arc<RTCPeerConnection>,
    pub(crate) data_channel: Arc<RTCDataChannel>,
    closed_reason: AtomicPtr<Option<anyhow::Error>>,
    closed: AtomicBool,
}

impl Debug for WebRTCBaseChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebRTCBaseChannel")
            .field("Peer connection id", &self.peer_connection.get_stats_id())
            .field("Data channel id", &self.data_channel.id())
            .finish()
    }
}

impl WebRTCBaseChannel {
    pub(crate) async fn new(
        peer_connection: Arc<RTCPeerConnection>,
        data_channel: Arc<RTCDataChannel>,
    ) -> Arc<Self> {
        let dc = data_channel.clone();
        let pc = Arc::downgrade(&peer_connection);
        peer_connection.on_ice_connection_state_change(Box::new(move |conn_state| {
            let pc = match pc.upgrade() {
                Some(pc) => pc,
                None => return Box::pin(async {}),
            };
            Box::pin(async move {
                let sctp = pc.sctp();
                let transport = sctp.transport();
                let transport = transport.ice_transport();
                let candidate_pair = transport.get_selected_candidate_pair().await;
                log::info!(
                    "Selected candidate pair. Pair: {:?}. ID: {}. Current connection state: {}",
                    candidate_pair,
                    conn_state,
                    pc.get_stats_id()
                );
            })
        }));

        let channel = Arc::new(Self {
            peer_connection,
            data_channel,
            closed_reason: AtomicPtr::new(&mut None),
            closed: AtomicBool::new(false),
        });

        let c = Arc::downgrade(&channel);
        dc.on_error(Box::new(move |err: webrtc::Error| {
            let c = match c.upgrade() {
                Some(c) => c,
                None => return Box::pin(async {}),
            };
            Box::pin(async move {
                let mut err = Some(anyhow::Error::from(err));
                c.closed_reason.store(&mut err, Ordering::Release);
                if let Err(e) = c.close_sync() {
                    log::error!("error closing channel: {e}")
                }
            })
        }));

        channel
    }

    /// Closes the channel
    #[allow(dead_code)]
    pub async fn close(&self) -> Result<()> {
        log::debug!("Closing base channel");
        if self.closed.load(Ordering::Acquire) {
            return Ok(());
        }
        self.closed.store(true, Ordering::Release);

        self.peer_connection
            .close()
            .await
            .map_err(anyhow::Error::from)
    }

    // `close` with blocking. Should only be used in contexts where an async close is disallowed
    pub(crate) fn close_sync(&self) -> Result<()> {
        set_hook(Box::new(|info| {
            log::error!(
        "Unable to close base_channel gracefully. This may be because of an attempt to connect to a robot that isn't online. Error message: {}",
        info.to_string()
        )
        }));
        let safe_self = AssertUnwindSafe(self);
        match catch_unwind(|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move { safe_self.close().await })
        }) {
            Ok(res) => res,
            Err(_) => Err(anyhow::anyhow!("Unable to close base channel gracefully")),
        }
    }

    /// Returns whether or not the channel is closed
    #[allow(dead_code)]
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Acquire)
    }
    /// Returns Some(reason) if the channel closed with error, otherwise None
    #[allow(dead_code)]
    pub fn closed_reason(&self) -> *mut Option<anyhow::Error> {
        self.closed_reason.load(Ordering::Acquire)
    }
}
