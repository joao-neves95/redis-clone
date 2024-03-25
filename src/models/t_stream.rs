use super::connection_context::Response;

use std::{fmt::Debug, net::SocketAddr};

use anyhow::Error;
use tokio::{
    io::{self, AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};

pub trait TStream: AsyncRead + AsyncWrite + Send + Unpin + Debug {
    fn local_addr(&self) -> io::Result<SocketAddr>;

    fn peer_addr(&self) -> io::Result<SocketAddr>;
}

impl TStream for TcpStream {
    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.local_addr()
    }

    fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.peer_addr()
    }
}

impl dyn TStream {
    pub async fn write_all_responses(&mut self, responses: &Vec<Response>) -> Result<(), Error> {
        for response in responses {
            println!("sending response - {:?}", response);

            let is_raw_response = response.command_byte_response.is_some();

            self.write_all(if !is_raw_response {
                response.command_response.as_bytes()
            } else {
                response.command_byte_response.as_ref().unwrap().as_slice()
            })
            .await?;

            self.flush().await?;
        }

        Ok(())
    }
}
