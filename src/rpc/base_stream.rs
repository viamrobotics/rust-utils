use crate::gen::proto::rpc::webrtc::v1::{PacketMessage, Stream};
use anyhow::Result;
use bytes::{BufMut, Bytes};
use http::HeaderMap;
use http_body::Frame;
use http_body_util::StreamBody;
use std::convert::Infallible;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

const MAX_MESSAGE_SIZE: usize = 1 << 25;

type BodyFrame = Result<Frame<Bytes>, Infallible>;

pub(crate) fn body_channel() -> (Sender, BodyReceiver) {
    let (tx, rx) = mpsc::channel(1);
    (Sender(tx), BodyReceiver(rx))
}

pub(crate) struct Sender(mpsc::Sender<BodyFrame>);

impl Sender {
    pub(crate) async fn send_data(&self, data: Bytes) -> Result<()> {
        self.send(Frame::data(data)).await
    }

    pub(crate) async fn send_trailers(&self, trailers: HeaderMap) -> Result<()> {
        self.send(Frame::trailers(trailers)).await
    }

    async fn send(&self, frame: Frame<Bytes>) -> Result<()> {
        self.0
            .send(Ok(frame))
            .await
            .map_err(|_| anyhow::anyhow!("response body was dropped"))
    }
}

// Held instead of a tonic::body::Body until the response is built, since tonic's body is !Sync
// and the client channel that stores it is shared across tasks.
pub(crate) struct BodyReceiver(mpsc::Receiver<BodyFrame>);

impl BodyReceiver {
    pub(crate) fn into_body(self) -> tonic::body::Body {
        tonic::body::Body::new(StreamBody::new(ReceiverStream::new(self.0)))
    }
}

/// Base elements of a webRTC stream, used in both client and server implementations
#[allow(dead_code)]
pub struct WebRTCBaseStream {
    pub(crate) stream: Stream,
    pub(crate) message_sender: Sender,
    pub(crate) closed: AtomicBool,
    pub(crate) packet_buffer: Vec<u8>,
    pub(crate) closed_reason: AtomicPtr<Option<anyhow::Error>>,
}

impl WebRTCBaseStream {
    pub(crate) fn close_with_recv_error(&self, err: &mut Option<&anyhow::Error>) {
        if self.closed.load(Ordering::Acquire) {
            return;
        }
        let mut err = err.map(|e| anyhow::anyhow!(e.to_string()));
        self.closed.store(true, Ordering::Release);
        self.closed_reason.store(&mut err, Ordering::Release);
    }

    pub(crate) fn process_message(&mut self, message: PacketMessage) -> Result<Option<Vec<u8>>> {
        if message.data.is_empty() && message.eom {
            return Ok(Some(Vec::new()));
        }
        if message.data.len() + self.packet_buffer.len() > MAX_MESSAGE_SIZE {
            let e = Err(anyhow::anyhow!(
                "message size larger than max {}, discarding",
                MAX_MESSAGE_SIZE
            ));
            self.reset_packet_buffer();
            return e;
        }

        self.packet_buffer.put_slice(&message.data);
        if message.eom {
            let ret = self.packet_buffer.clone();
            self.reset_packet_buffer();
            return Ok(Some(ret));
        }
        Ok(None)
    }

    fn reset_packet_buffer(&mut self) {
        self.packet_buffer = vec![]
    }
}
