use hyper::server::accept::Accept;
use std::io::Error;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::net::{TcpListener, TcpStream};

pub struct Connector {
    inner: TcpListener,
    addr: String,
}

impl Connector {
    pub fn new() -> Result<Self, Error> {
        let tcp = std::net::TcpListener::bind("127.0.0.1:0")?;
        tcp.set_nonblocking(true)?;
        Ok(Connector {
            addr: tcp.local_addr()?.to_string(),
            inner: TcpListener::from_std(tcp)?,
        })
    }
    pub fn get_path(&self) -> &str {
        &self.addr
    }
}

impl Accept for Connector {
    type Conn = TcpStream;
    type Error = Error;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        match self.inner.poll_accept(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok((stream, _port))) => Poll::Ready(Some(Ok(stream))),
            Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
        }
    }
}
