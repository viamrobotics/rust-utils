use super::{base_channel::*, base_stream::*, client_stream::*};
use crate::gen::proto::rpc::webrtc::v1::{
    request::Type, response::Type as RespType, PacketMessage, Request, RequestHeaders,
    RequestMessage, Response, Stream,
};
use anyhow::Result;
use dashmap::DashMap;
use hyper::Body;
use prost::Message;
use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering},
        Arc, RwLock,
    },
};
use webrtc::{
    data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    peer_connection::RTCPeerConnection,
};

// see golang/client_stream.go
const MAX_REQUEST_MESSAGE_PACKET_DATA_SIZE: usize = 16373;
// 256 is an arbitrarily high number for maximum concurrent streams, determined based on
// analogous value in goutils
const MAX_CONCURRENT_STREAM_COUNT: usize = 256;

/// The client-side implementation of a webRTC connection channel.
pub struct WebRTCClientChannel {
    pub(crate) base_channel: Arc<WebRTCBaseChannel>,
    stream_id_counter: AtomicU64,
    pub(crate) streams: DashMap<u64, WebRTCClientStream>,
    pub(crate) receiver_bodies: DashMap<u64, hyper::Body>,
    // String type rather than error type because anyhow::Error does not derive clone
    pub(crate) error: RwLock<Option<String>>,
}

impl Debug for WebRTCClientChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebRTCClientChannel")
            .field("stream_id_counter", &self.stream_id_counter)
            .field("base channel", &self.base_channel)
            .finish()
    }
}

impl Drop for WebRTCClientChannel {
    fn drop(&mut self) {
        log::debug!("Dropping client channel {:?}", &self);
    }
}

impl WebRTCClientChannel {
    pub async fn close(&self) {
        self.base_channel.close().await.unwrap();
        self.base_channel.data_channel.close().await.unwrap();
        self.base_channel.peer_connection.close().await.unwrap();
    }

    pub(crate) async fn new(
        peer_connection: Arc<RTCPeerConnection>,
        data_channel: Arc<RTCDataChannel>,
    ) -> Arc<Self> {
        let base_channel = WebRTCBaseChannel::new(peer_connection, data_channel.clone()).await;
        let error = RwLock::new(None);
        let channel = Self {
            error,
            base_channel,
            streams: DashMap::new(),
            stream_id_counter: AtomicU64::new(0),
            receiver_bodies: DashMap::new(),
        };

        let channel = Arc::new(channel);
        let ret_channel = channel.clone();
        let channel = Arc::downgrade(&channel);

        data_channel.on_message(Box::new(move |msg: DataChannelMessage| {
            let channel = channel.clone();
            Box::pin(async move {
                let channel = match channel.upgrade() {
                    Some(channel) => channel,
                    None => {
                        return;
                    }
                };
                let maybe_err = channel.on_channel_message(msg).await;
                let mut err = channel.error.write().unwrap();
                match maybe_err {
                    Err(e) => {
                        log::error!("error deserializing message: {e}");
                        *err = Some(e.to_string());
                    }
                    Ok(()) => *err = None,
                }
            })
        }));
        log::debug!("Client channel created");
        ret_channel
    }

    pub(crate) fn new_stream(&self) -> Result<Stream> {
        if self.streams.len() >= MAX_CONCURRENT_STREAM_COUNT {
            return Err(anyhow::anyhow!(
                "Reached max concurrent stream cap of {MAX_CONCURRENT_STREAM_COUNT}; unable to add new stream."
            ));
        }
        let id = self.stream_id_counter.fetch_add(1, Ordering::AcqRel);
        let stream = Stream { id };
        let (message_sender, receiver_body) = hyper::Body::channel();

        let base_stream = WebRTCBaseStream {
            stream: stream.clone(),
            message_sender,
            closed: AtomicBool::new(false),
            packet_buffer: Vec::new(),
            closed_reason: AtomicPtr::new(&mut None),
        };

        let client_stream = WebRTCClientStream {
            base_stream,
            headers_received: AtomicBool::new(false),
            trailers_received: AtomicBool::new(false),
        };

        let _ = self.streams.insert(id, client_stream);
        let _ = self.receiver_bodies.insert(id, receiver_body);
        Ok(stream)
    }

    async fn on_channel_message(&self, msg: DataChannelMessage) -> Result<()> {
        let response = Response::decode(&*msg.data.to_vec())?;
        let (active_stream, stream_id) = match response.stream.as_ref() {
            None => {
                log::error!(
                    "no stream associated with response {:?}: discarding response",
                    response
                );
                return Ok(());
            }
            Some(stream) => {
                let id: u64 = stream.id;
                let stream = self.streams.get_mut(&stream.id).ok_or_else(|| {
                    anyhow::anyhow!(
                        "No stream found for id {}: discarding response {:?}",
                        &stream.id,
                        response
                    )
                });
                (stream, id)
            }
        };

        let should_drop_stream = matches!(response.r#type, Some(RespType::Trailers(_)));

        let maybe_err = match active_stream {
            Ok(mut active_stream) => active_stream.on_response(response).await,
            Err(e) => Err(anyhow::anyhow!("Error acquiring active stream: {e}")),
        };

        if should_drop_stream {
            self.streams.remove(&stream_id);
        }
        maybe_err
    }

    pub(crate) fn resp_body_from_stream(&self, stream_id: u64) -> Result<Body> {
        match self.receiver_bodies.remove(&stream_id) {
            Some(entry) => Ok(entry.1),
            None => Err(anyhow::anyhow!(
                "Tried to receive stream {stream_id} but it didn't exist!"
            )),
        }
    }

    pub(crate) async fn write_headers(
        &self,
        stream: &Stream,
        headers: RequestHeaders,
    ) -> Result<()> {
        let headers = Request {
            stream: Some(stream.clone()),
            r#type: Some(Type::Headers(headers)),
        };
        let header_vec = Message::encode_to_vec(&headers);
        self.send(&header_vec).await
    }

    /// Sends a single complete gRPC-framed message (1-byte compression flag + 4-byte
    /// big-endian length + payload), packetizing the payload across multiple Requests
    /// when it exceeds MAX_REQUEST_MESSAGE_PACKET_DATA_SIZE. The caller is responsible
    /// for splitting a body into individual gRPC messages and for signaling end of
    /// stream via `write_eos` once all messages have been sent.
    pub(crate) async fn write_grpc_message(&self, stream: &Stream, data: &[u8]) -> Result<()> {
        if data.len() < 5 {
            return Err(anyhow::anyhow!(
                "Attempted to process message with irregular length"
            ));
        }

        // 1..5 are the gRPC length-prefix bytes; strip the 5-byte header and packetize
        // the payload. The webrtc eom/eos framing replaces the gRPC framing on the wire.
        let mut payload = &data[5..];
        loop {
            let split_at = MAX_REQUEST_MESSAGE_PACKET_DATA_SIZE.min(payload.len());
            let (to_send, remaining) = payload.split_at(split_at);
            let eom = remaining.is_empty();
            let request = Request {
                stream: Some(stream.clone()),
                r#type: Some(Type::Message(RequestMessage {
                    has_message: true,
                    eos: false,
                    packet_message: Some(PacketMessage {
                        eom,
                        data: to_send.to_vec(),
                    }),
                })),
            };

            let request = Message::encode_to_vec(&request);
            if let Err(e) = self.send(&request).await {
                log::error!("error sending message: {e}");
                return Err(e);
            }

            payload = remaining;
            if eom {
                break;
            }
        }
        Ok(())
    }

    /// Signals the end of the client's send stream (the gRPC CloseSend equivalent).
    // note(ethan): the variable that used to carry this signal was named
    // `it_was_all_a_stream` — the best variable name I've ever shipped into production.
    // Memorialized here so it is not forgotten.
    pub(crate) async fn write_eos(&self, stream: &Stream) -> Result<()> {
        let request = Request {
            stream: Some(stream.clone()),
            r#type: Some(Type::Message(RequestMessage {
                has_message: false,
                eos: true,
                packet_message: None,
            })),
        };
        self.send(&Message::encode_to_vec(&request)).await
    }

    async fn send(&self, data: &[u8]) -> Result<()> {
        let data = &bytes::Bytes::copy_from_slice(data);
        self.base_channel
            .data_channel
            .send(data)
            .await
            .map_err(anyhow::Error::from)
            .map(|_: usize| ())
    }

    pub(crate) fn close_stream_with_recv_error(&self, stream_id: u64, error: anyhow::Error) {
        match self.streams.remove(&stream_id) {
            Some(entry) => entry.1.base_stream.close_with_recv_error(&mut Some(&error)),
            None => {
                log::error!("attempted to close stream with id {stream_id}, but it wasn't found!")
            }
        }
    }

    /// Returns the current stats report associated with the underlying peer connection.
    pub async fn get_stats(&self) -> webrtc::stats::StatsReport {
        self.base_channel.peer_connection.get_stats().await
    }
}
