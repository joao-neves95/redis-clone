#[cfg(test)]
pub(crate) mod utils {
    use crate::{
        models::{
            db::{app_data::AppData, in_memory_db::InMemoryDb},
            t_stream::TStream,
        },
        DEFAULT_LISTENING_PORT,
    };

    use std::{
        net::{Ipv4Addr, SocketAddr, SocketAddrV4},
        sync::Arc,
        task::Poll,
    };

    use anyhow::Error;
    use tokio::{
        io::{AsyncRead, AsyncWrite},
        sync::Mutex,
    };

    pub(crate) fn create_test_mem_db<'a>() -> Result<Arc<Mutex<InMemoryDb>>, Error> {
        Ok(InMemoryDb::new(AppData::new_master(
            DEFAULT_LISTENING_PORT,
        )?)?)
    }

    pub(crate) fn create_test_tstream() -> Arc<Mutex<dyn TStream>> {
        Arc::new(Mutex::new(FakeTStream {}))
    }

    pub(crate) fn create_fake_socket_addr() -> std::net::SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(127, 0, 0, 1),
            DEFAULT_LISTENING_PORT,
        ))
    }

    #[derive(Debug)]
    pub(crate) struct FakeTStream {}

    impl TStream for FakeTStream {
        fn local_addr(&self) -> tokio::io::Result<SocketAddr> {
            Ok(create_fake_socket_addr())
        }

        fn peer_addr(&self) -> tokio::io::Result<std::net::SocketAddr> {
            Ok(create_fake_socket_addr())
        }
    }

    impl AsyncWrite for FakeTStream {
        fn poll_write(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
            _: &[u8],
        ) -> std::task::Poll<Result<usize, std::io::Error>> {
            Poll::Ready(Ok(0))
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), std::io::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), std::io::Error>> {
            Poll::Ready(Ok(()))
        }
    }

    impl AsyncRead for FakeTStream {
        fn poll_read(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
            _: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }
}
