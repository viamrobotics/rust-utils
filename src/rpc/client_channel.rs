use super::{base_channel::*, base_stream::*, client_stream::*};
use crate::gen::proto::rpc::webrtc::v1::{
    request::Type, response::Type as RespType, PacketMessage, Request, RequestHeaders,
    RequestMessage, Response, Stream,
};
use anyhow::Result;
use chashmap::CHashMap;
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

/// The client-side implementation of a webRTC connection channel.
pub struct WebRTCClientChannel {
    pub(crate) base_channel: Arc<WebRTCBaseChannel>,
    stream_id_counter: AtomicU64,
    pub(crate) streams: CHashMap<u64, WebRTCClientStream>,
    pub(crate) receiver_bodies: CHashMap<u64, hyper::Body>,
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
        let bc = self.base_channel.clone();
        if !bc.is_closed() {
            let _ = tokio::spawn(async move {
                if let Err(e) = bc.close().await {
                    log::error!("Error closing base channel: {e}");
                }
            });
        };
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
            streams: CHashMap::new(),
            stream_id_counter: AtomicU64::new(0),
            receiver_bodies: CHashMap::new(),
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

    pub(crate) fn new_stream(&self) -> Stream {
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
        stream
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
        self.receiver_bodies
            .remove(&stream_id)
            .ok_or(anyhow::anyhow!(
                "Tried to receive stream {stream_id} but it didn't exist!"
            ))
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

    pub(crate) async fn write_message(
        &self,
        eos: bool,
        stream: Option<Stream>,
        mut data: Vec<u8>,
    ) -> Result<()> {
        // even if no meaningful data, any actual message will include at least frame header bytes
        let has_message = !data.is_empty();

        // rust libraries are munging streamed client requests into a single http request.
        // we can look at the gRPC header bytes to determine the length of the first message
        // and compare it to the length of the data to determine whether this http request
        // is a single unary call, or a streaming call.
        // TODO(RSDK-654) The munging of streaming requests into a single http request is
        // likely going to cause problems for us when we encounter a need for bidi streaming
        // in the real world. Look into how we can fix it, and hopefully get rid of this
        // header math in the process.

        let mut to_add_bytes = [0u8; 4];
        // 1-5 because those are the length header bytes for gRPC
        to_add_bytes.clone_from_slice(&data[1..5]);
        let mut next_message_length = u32::from_be_bytes(to_add_bytes);
        // if this is all streaming calls we need to tell the server when we're done with
        // the stream, otherwise neither side will know we're done, trailers will never be
        // sent/processed, and we'll hang on the strream.
        let it_was_all_a_stream = usize::try_from(next_message_length).unwrap() + 5 < data.len();

        // always run the loop at least once, check at completion if we've sent all data and
        // break the loop accordingly
        loop {
            if data.len() < 5 {
                return Err(anyhow::anyhow!(
                    "Attempted to process message with irregular length"
                ));
            }

            // because we might have multiple requests contained within our data, we have
            // to do the manual work of breaking apart the body into separate requests.
            to_add_bytes.clone_from_slice(&data[1..5]);
            next_message_length = u32::from_be_bytes(to_add_bytes);
            data = data.split_off(5);
            // we need an internal loop because a single message may be longer than the
            // MAX_REQUEST_MESSAGE_PACKET_DATA_SIZE in which case we don't want to shave off
            // a five byte header. but, a single call to write_message may contain multiple
            // distinct messages within the data vec, so we want to be able to evaluate length
            // multiple times if necessary. we use a loop with an exit check at the bottom
            // because we always want to send a request at least once, even if the data is empty.
            loop {
                let split_at = MAX_REQUEST_MESSAGE_PACKET_DATA_SIZE
                    .min(data.len())
                    .min(usize::try_from(next_message_length).unwrap());
                let (to_send, remaining) = data.split_at(split_at);
                next_message_length -= u32::try_from(split_at).unwrap();
                let stream = stream.clone();
                let request = Request {
                    stream,
                    r#type: Some(Type::Message(RequestMessage {
                        has_message,
                        eos: if !remaining.is_empty() {
                            // stream definitely isn't done if there's more to send
                            false
                        } else {
                            // if we intentionally sent an eos or the http request was inferrably
                            // a stream
                            eos || it_was_all_a_stream
                        },
                        packet_message: Some(PacketMessage {
                            eom: next_message_length == 0 || remaining.is_empty(),
                            data: to_send.to_vec(),
                        }),
                    })),
                };

                let request = Message::encode_to_vec(&request);
                if let Err(e) = self.send(&request).await {
                    log::error!("error sending message: {e}");
                    return Err(e);
                }

                data = remaining.to_vec();
                if next_message_length == 0 {
                    break;
                }
            }
            if data.is_empty() {
                break;
            }
        }
        Ok(())
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
            Some(stream) => stream.base_stream.close_with_recv_error(&mut Some(&error)),
            None => {
                log::error!("attempted to close stream with id {stream_id}, but it wasn't found!")
            }
        }
    }
}
