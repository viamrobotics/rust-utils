use anyhow::Result;
use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, AtomicPtr, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};
use webrtc::{data_channel::RTCDataChannel, peer_connection::RTCPeerConnection};

// see golang/client_stream.go
/// The base components to a webRTC channel, used on both client and server sides.
pub struct WebRTCBaseChannel {
    pub(crate) peer_connection: Arc<RTCPeerConnection>,
    pub(crate) data_channel: Arc<RTCDataChannel>,
    closed_reason: AtomicPtr<Option<anyhow::Error>>,
    closed: AtomicBool,
    pub(crate) should_close: AtomicBool,
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
    async fn close_loop(c: std::sync::Weak<Self>) {
        let c = match c.upgrade() {
            Some(c) => c,
            None => return (),
        };
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            loop {
                sleep(Duration::from_millis(100));
                if !c.should_close.load(Ordering::Acquire) {
                    continue;
                }
                if let Err(e) = c.close_with_reason().await {
                    log::error!("error closing channel: {e}")
                }
                break;
            }
        })
    }

    async fn new_(
        peer_connection: Arc<RTCPeerConnection>,
        data_channel: Arc<RTCDataChannel>,
    ) -> Arc<Self> {
        let dc = data_channel.clone();
        let pc = Arc::downgrade(&peer_connection);
        peer_connection
            .on_ice_connection_state_change(Box::new(move |conn_state| {
                let pc = match pc.upgrade(){
                    Some(pc) => pc,
                    None => return Box::pin(async  {}),
                };
                Box::pin(async move {
                    let sctp = pc.sctp();
                    let transport = sctp.transport();
                    let transport = transport.ice_transport();
                    let candidate_pair = transport.get_selected_candidate_pair().await;
                    log::info!(
                        "Selected candidate pair. Pair: {candidate_pair:?}. ID: {}. Current connection state: {conn_state}",
                        pc.get_stats_id()
                    );
                })
            }));

        let channel = Arc::new(Self {
            peer_connection,
            data_channel,
            closed_reason: AtomicPtr::new(&mut None),
            closed: AtomicBool::new(false),
            should_close: AtomicBool::new(false),
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
                c.should_close.store(true, Ordering::Release);
                // CR erodkin: clean up!
                //Box::pin(async move {
                //if let Err(e) = c.close_with_reason(Some(anyhow::Error::from(err))).await {
                //log::error!("error closing channel: {e}")
                //}
            })
        }));

        channel
    }

    pub(crate) async fn new(
        peer_connection: Arc<RTCPeerConnection>,
        data_channel: Arc<RTCDataChannel>,
    ) -> Arc<Self> {
        let chan = Self::new_(peer_connection, data_channel).await;
        let c = Arc::downgrade(&chan);
        tokio::task::spawn(async move { Self::close_loop(c) });
        chan
    }

    async fn close_with_reason(&self) -> Result<()> {
        if self.closed.load(Ordering::Acquire) {
            return Ok(());
        }
        self.closed.store(true, Ordering::Release);

        self.peer_connection
            .close()
            .await
            .map_err(anyhow::Error::from)
    }

    /// Closes the channel
    #[allow(dead_code)]
    pub async fn close(&self) -> Result<()> {
        self.close_with_reason().await
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
