use crate::{
    models::{
        app_context::{AppContext, Request},
        db::InMemoryDb,
    },
    resp_parser::{
        self,
        shared::{RespCommandResponseNames, RespDataTypesFirstByte},
    },
    TCP_READ_TIMEOUT,
};

use std::sync::Arc;

use anyhow::{Error, Ok, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

pub(crate) async fn run(mem_db: &Arc<Mutex<InMemoryDb>>) -> Result<(), Error> {
    println!("running handshake");

    let (master_host, master_port) = {
        let mem_db_lock = mem_db.lock().await;
        let replica_config = mem_db_lock.get_app_data_ref().replica.as_ref().unwrap();

        (
            replica_config.master_host.clone(),
            replica_config.master_port,
        )
    };

    let mut tcp_stream = TcpStream::connect(format!("{}:{}", master_host, master_port)).await?;

    send_ping(mem_db, &mut tcp_stream).await?;
    send_replconf(mem_db, &mut tcp_stream).await?;
    send_psync(mem_db, &mut tcp_stream).await?;

    println!("finished handshake.");
    Ok(())
}

async fn send_ping(
    mem_db: &Arc<Mutex<InMemoryDb>>,
    tcp_stream: &mut TcpStream,
) -> Result<(), Error> {
    println!("sending PING");

    tcp_stream.write_all(b"*1\r\n$4\r\nping\r\n").await?;
    tcp_stream.flush().await?;

    println!("awaiting PONG as response...");

    await_response(mem_db, tcp_stream, |response| {
        response.as_str() == RespCommandResponseNames::PONG
    })
    .await?;

    println!("PONG obtained");

    Ok(())
}

async fn send_replconf(
    mem_db: &Arc<Mutex<InMemoryDb>>,
    tcp_stream: &mut TcpStream,
) -> Result<(), Error> {
    println!("sending REPLCONF 1 (listening-port).");
    tcp_stream
        .write_all(b"*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n$4\r\n6380\r\n")
        .await?;
    tcp_stream.flush().await?;

    await_response_ok(mem_db, tcp_stream).await?;

    println!("sending REPLCONF 2 (capabilities).");
    tcp_stream
        .write_all(b"*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n")
        .await?;
    tcp_stream.flush().await?;

    await_response_ok(mem_db, tcp_stream).await?;

    Ok(())
}

async fn send_psync(
    mem_db: &Arc<Mutex<InMemoryDb>>,
    tcp_stream: &mut TcpStream,
) -> Result<(), Error> {
    println!("sending PSYNC (synchronize state)");
    tcp_stream
        .write_all(b"*3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n")
        .await?;
    tcp_stream.flush().await?;

    println!("awaiting FULLRESYNC as response...");

    await_response(mem_db, tcp_stream, |response| {
        response.starts_with("FULLRESYNC")
    })
    .await?;

    println!("FULLRESYNC obtained.");

    Ok(())
}

async fn await_response_ok(
    mem_db: &Arc<Mutex<InMemoryDb>>,
    tcp_stream: &mut TcpStream,
) -> Result<(), Error> {
    println!("awaiting OK as response...");

    await_response(mem_db, tcp_stream, |response| {
        response.as_str() == RespCommandResponseNames::OK
    })
    .await?;

    println!("OK obtained.");

    Ok(())
}

async fn await_response(
    mem_db: &Arc<Mutex<InMemoryDb>>,
    tcp_stream: &mut TcpStream,
    expected_response_predicate: impl Fn(&String) -> bool,
) -> Result<(), Error> {
    let mut response = String::new();

    println!("listening for requests...");

    while !expected_response_predicate(&response) {
        let mut request_buffer: [u8; 1024] = [0; 1024];

        match tokio::time::timeout(TCP_READ_TIMEOUT, tcp_stream.read(&mut request_buffer)).await {
            Err(e) => {
                println!("timeout while reading request - {}", e);
                break;
            }
            Result::Ok(read_result) => {
                let request_len = match read_result {
                    Err(e) => {
                        const BASE_MESSAGE: &str = "Error while reading handshake response";
                        println!("{} - {:?}", BASE_MESSAGE, e);

                        return Err(Error::msg(format!("{}: timeout.", BASE_MESSAGE)));
                    }
                    Result::Ok(len) => len,
                };

                println!("request received of len {}", request_len);
                // println!(
                //     "request received of len {} (is_ascii - {}) - {:?}",
                //     request_len,
                //     request_buffer.is_ascii(),
                //     String::from_utf8_lossy(&request_buffer)
                // );

                if request_len == 0 {
                    break;
                }

                let mut context = AppContext::new_request(
                    mem_db,
                    if request_buffer.starts_with(&[RespDataTypesFirstByte::SIMPLE_STRINGS_BYTE]) {
                        Request::new_simple_string(&mut request_buffer)?
                    } else {
                        Request::new(&request_buffer)?
                    },
                )?;

                response =
                    resp_parser::parse_redis_resp_proc_response(&mut context)?.get_value_string();
            }
        };
    }

    Ok(())
}
