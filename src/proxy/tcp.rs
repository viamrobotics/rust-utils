use std::io::Error;

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
    pub async fn accept(&self) -> Result<TcpStream, Error> {
        let (stream, _port) = self.inner.accept().await?;
        Ok(stream)
    }
}
