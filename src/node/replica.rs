use crate::{
    models::{connection_context::InternalRequest, db::in_memory_db::InMemoryDb},
    resp_parser::{
        self,
        shared::{RespCommandResponseNames, RespDataTypesFirstByte},
    },
    TCP_READ_TIMEOUT, TCP_READ_TIMEOUT_MAX_RETRIES, TCP_RESPONSE_BUFFER_SIZE,
};

use std::sync::Arc;

use anyhow::{Error, Ok, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

pub(crate) async fn handshake<'a>(mem_db: &Arc<Mutex<InMemoryDb>>) -> Result<(), Error> {
    println!("running handshake");

    let (master_host, master_port) = {
        let mem_db_lock = mem_db.lock().await;
        let replica_config = mem_db_lock
            .get_app_data_ref()
            .get_replication_data_ref()
            .unwrap();
        (
            replica_config.master_host.clone(),
            replica_config.master_port,
        )
    };

    let mut tcp_stream_with_master =
        TcpStream::connect(format!("{}:{}", master_host, master_port)).await?;

    send_ping(&mut tcp_stream_with_master).await?;
    send_replconf(&mut tcp_stream_with_master).await?;
    send_psync(&mut tcp_stream_with_master).await?;

    println!("finished handshake.");

    Ok(())
}

async fn send_ping(tcp_stream: &mut TcpStream) -> Result<(), Error> {
    println!("sending PING");

    tcp_stream.write_all(b"*1\r\n$4\r\nping\r\n").await?;
    tcp_stream.flush().await?;

    println!("awaiting PONG as response...");

    await_response(tcp_stream, |response| {
        response.as_str() == RespCommandResponseNames::PONG
    })
    .await?;

    println!("PONG obtained");

    Ok(())
}

async fn send_replconf(tcp_stream: &mut TcpStream) -> Result<(), Error> {
    println!("sending REPLCONF 1 (listening-port).");
    tcp_stream
        .write_all(b"*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n$4\r\n6380\r\n")
        .await?;
    tcp_stream.flush().await?;

    await_response_ok(tcp_stream).await?;

    println!("sending REPLCONF 2 (capabilities).");
    tcp_stream
        .write_all(b"*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n")
        .await?;
    tcp_stream.flush().await?;

    await_response_ok(tcp_stream).await?;

    Ok(())
}

async fn send_psync(tcp_stream: &mut TcpStream) -> Result<(), Error> {
    println!("sending PSYNC (synchronize state)");
    tcp_stream
        .write_all(b"*3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n")
        .await?;
    tcp_stream.flush().await?;

    println!("awaiting FULLRESYNC as response...");

    await_response(tcp_stream, |response| response.starts_with("FULLRESYNC")).await?;

    println!("FULLRESYNC obtained.");

    Ok(())
}

async fn await_response_ok(tcp_stream: &mut TcpStream) -> Result<(), Error> {
    println!("awaiting OK as response...");

    await_response(tcp_stream, |response| {
        response.as_str() == RespCommandResponseNames::OK
    })
    .await?;

    println!("OK obtained.");

    Ok(())
}

async fn await_response(
    tcp_stream: &mut TcpStream,
    expected_response_predicate: impl Fn(&String) -> bool,
) -> Result<(), Error> {
    let mut response = String::new();
    let mut num_of_retries = 0;

    println!("listening for requests...");

    while num_of_retries <= TCP_READ_TIMEOUT_MAX_RETRIES && !expected_response_predicate(&response)
    {
        num_of_retries += 1;
        let mut request_buffer = [0; TCP_RESPONSE_BUFFER_SIZE];

        match tokio::time::timeout(TCP_READ_TIMEOUT, tcp_stream.read(&mut request_buffer)).await {
            Err(e) => {
                println!("timeout while reading request - {}", e);
                break;
            }
            Result::Ok(read_result) => {
                let request_byte_count = match read_result {
                    Err(e) => {
                        const BASE_MESSAGE: &str = "Error while reading handshake response";
                        println!("{} - {:?}", BASE_MESSAGE, e);

                        return Err(Error::msg(format!("{}: timeout.", BASE_MESSAGE)));
                    }
                    Result::Ok(len) => len,
                };

                println!("request received of len {}", request_byte_count);

                if request_byte_count == 0 {
                    break;
                }

                response = resp_parser::parse_redis_resp_proc_response(&InternalRequest {
                    buffer: request_buffer,
                    byte_count: request_byte_count,
                })?
                .get_value_string();
            }
        };
    }

    Ok(())
}
