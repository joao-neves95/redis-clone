use crate::{
    models::{
        connection_context::{ConnectionContext, Handshake},
        db::in_memory_db::InMemoryDb,
        t_stream::TStream,
    },
    node::{command_handlers, propagation::propagate},
    resp_parser::{self, shared::RespCommandNames},
    TCP_READ_TIMEOUT,
};

use std::sync::Arc;

use anyhow::Error;
use tokio::{io::AsyncReadExt, net::TcpListener, sync::Mutex};

pub(crate) async fn run<'a>(mem_db: &Arc<Mutex<InMemoryDb>>) -> Result<(), Error> {
    let listening_port = {
        let db_lock = mem_db.lock().await;
        db_lock.get_app_data_ref().listening_port
    };

    let listener = TcpListener::bind(format!("127.0.0.1:{}", listening_port)).await?;

    loop {
        let _ = match listener.accept().await {
            Ok((mut _tcp_stream, addy)) => {
                let mem_db_arc_pointer = Arc::clone(mem_db);
                let tcp_stream_arc: Arc<Mutex<dyn TStream>> = Arc::new(Mutex::new(_tcp_stream));

                tokio::spawn(async move {
                    let mut connection_context =
                        ConnectionContext::new(&mem_db_arc_pointer, &tcp_stream_arc).unwrap();

                    connection_context
                        .println_by(&format!(
                            "accepted new connection from {}:{}",
                            addy.ip(),
                            addy.port()
                        ))
                        .await;

                    match handle_client_connection(&mut connection_context).await {
                        Err(e) => {
                            connection_context
                                .println_by(&format!("connection handling error: {}", e))
                                .await;
                        }
                        Ok(()) => (),
                    }

                    connection_context
                        .println_by("finished handling request, closing tcp stream")
                        .await;

                    mem_db_arc_pointer
                        .lock()
                        .await
                        .get_app_data_mut()
                        .get_master_data_mut()
                        .unwrap()
                        .slaves
                        .remove(&match connection_context.request.handshake {
                            Handshake::Replica { port } => port,
                            Handshake::None => 0,
                        });
                });
            }
            Err(e) => {
                println!("tcp connection error: {}", e);
            }
        };
    }
}

async fn handle_client_connection<'a>(
    connection_context: &mut ConnectionContext<'a>,
) -> Result<(), anyhow::Error> {
    connection_context
        .println_by("listening for requests on this stream...")
        .await;

    loop {
        connection_context.reset();

        match tokio::time::timeout(TCP_READ_TIMEOUT, async {
            let mut tcp_stream_lock = connection_context.request.tcp_stream.lock().await;
            tcp_stream_lock
                .read(&mut connection_context.request.buffer)
                .await
        })
        .await
        {
            Err(_) => {
                connection_context
                    .println_by("timeout, waiting for a new request on this stream...")
                    .await;

                continue;
            }
            Ok(read_result) => {
                let request_byte_count = read_result?;

                connection_context
                    .println_by(&format!("request received of len {}", request_byte_count))
                    .await;

                if request_byte_count == 0 {
                    // The socket is closed.
                    connection_context
                        .println_by(&format!("0 byte request, close tcp connection."))
                        .await;

                    break;
                }

                connection_context.request.byte_count = request_byte_count;

                handle_command(connection_context).await?;

                connection_context
                    .println_by(&format!(
                        "responding to request with: {:?}",
                        &connection_context.response
                    ))
                    .await;

                connection_context
                    .request
                    .tcp_stream
                    .lock()
                    .await
                    .write_all_responses(&connection_context.response)
                    .await?;

                propagate(connection_context).await?;
            }
        };

        connection_context
            .println_by("finished handling request, waiting for moar...")
            .await;
    }

    Ok(())
}

async fn handle_command<'a>(app_context: &mut ConnectionContext<'a>) -> Result<(), anyhow::Error> {
    app_context.println_by("parsing request").await;

    resp_parser::parse_resp_proc_command(app_context)?;

    app_context
        .println_by(&format!(
            "handling request - {}",
            app_context.format_request_info(true)?
        ))
        .await;

    match app_context
        .get_request_resp_command_ref()
        .unwrap()
        .name
        .as_str()
    {
        RespCommandNames::PING => command_handlers::handle_command_ping(app_context)?,
        RespCommandNames::REPLCONF => {
            command_handlers::handle_command_replconf(app_context).await?
        }
        RespCommandNames::PSYNC => command_handlers::handle_command_psync(app_context).await?,
        RespCommandNames::ECHO => command_handlers::handle_command_echo(app_context)?,
        RespCommandNames::GET => command_handlers::handle_command_get_async(app_context).await?,
        RespCommandNames::SET => command_handlers::handle_command_set_async(app_context).await?,
        RespCommandNames::INFO => command_handlers::handle_command_info(app_context).await?,

        _ => {
            return Err(Error::msg(
                "Could not handle command - Unknown or not implemented command.",
            ))
        }
    };

    app_context.println_by("finished handling request.").await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        models::connection_context::ConnectionContext,
        node::{
            command_handlers::{
                handle_command_echo, handle_command_get_async, handle_command_ping,
                handle_command_set_async,
            },
            command_listener::handle_command,
        },
        resp_parser::{parse_resp_proc_command, shared::RespCommandNames},
        test_helpers::utils::{create_test_mem_db, create_test_tstream},
        utils::copy_to_array_until,
    };

    use anyhow::Ok;

    #[tokio::test]
    async fn handle_command_handles_ping() -> Result<(), anyhow::Error> {
        let fake_mem_db = create_test_mem_db()?;
        let mut fake_tcp_stream = create_test_tstream();

        let mut fake_app_context = ConnectionContext::new(&fake_mem_db, &mut fake_tcp_stream)?;
        let request_buffer = b"*1\r\n$4\r\npiNg\r\n";
        copy_to_array_until(
            &mut fake_app_context.request.buffer,
            request_buffer,
            0,
            |_, _, source_idx| source_idx == request_buffer.len() - 1,
        );
        fake_app_context.request.byte_count = request_buffer.len();

        parse_resp_proc_command(&mut fake_app_context)?;
        assert_eq!(
            fake_app_context
                .get_request_resp_command_ref()
                .unwrap()
                .name,
            RespCommandNames::PING
        );
        assert_eq!(
            fake_app_context
                .get_request_resp_command_ref()
                .unwrap()
                .parameters
                .len(),
            0
        );

        handle_command_ping(&mut fake_app_context)?;
        handle_command(&mut fake_app_context).await?;
        assert_eq!(
            fake_app_context
                .response
                .first()
                .unwrap()
                .command_response
                .to_owned(),
            "+PONG\r\n".to_owned()
        );
        assert_eq!(
            fake_app_context.response.first().unwrap().command_response,
            "+PONG\r\n".to_owned()
        );

        Ok(())
    }

    #[tokio::test]
    async fn handle_command_handles_echo() -> Result<(), anyhow::Error> {
        let fake_mem_db = create_test_mem_db()?;
        let mut fake_tcp_stream = create_test_tstream();

        let mut fake_app_context = ConnectionContext::new(&fake_mem_db, &mut fake_tcp_stream)?;
        let request_buffer = b"*2\r\n$4\r\nEcHo\r\n$19\r\nHey world, I'm Joe!\r\n";
        copy_to_array_until(
            &mut fake_app_context.request.buffer,
            request_buffer,
            0,
            |_, _, source_idx| source_idx == request_buffer.len() - 1,
        );
        fake_app_context.request.byte_count = request_buffer.len();

        parse_resp_proc_command(&mut fake_app_context)?;
        assert_eq!(
            fake_app_context
                .get_request_resp_command_ref()
                .unwrap()
                .name,
            RespCommandNames::ECHO
        );
        assert_eq!(
            fake_app_context
                .get_request_resp_command_ref()
                .unwrap()
                .parameters
                .len(),
            1
        );

        handle_command_echo(&mut fake_app_context)?;
        handle_command(&mut fake_app_context).await?;
        assert_eq!(
            fake_app_context
                .response
                .first()
                .unwrap()
                .command_response
                .to_owned(),
            "$19\r\nHey world, I'm Joe!\r\n".to_owned()
        );
        assert_eq!(
            fake_app_context.response.first().unwrap().command_response,
            "$19\r\nHey world, I'm Joe!\r\n".to_owned()
        );

        Ok(())
    }

    #[tokio::test]
    async fn handle_command_handles_set_get() -> Result<(), anyhow::Error> {
        let fake_mem_db = create_test_mem_db()?;
        let mut fake_tcp_stream = create_test_tstream();

        // Set:
        let mut fake_app_context_set = ConnectionContext::new(&fake_mem_db, &mut fake_tcp_stream)?;
        let request_buffer_set = b"*3\r\n$3\r\nsET\r\n$3\r\nfoo\r\n$19\r\nHey world, I'm Joe!\r\n";
        // let request_buffer_set = b"*3\r\n$3\r\nsET\r\n$3\r\nfoo\r\n$19\r\nHey world, I'm Joe!\r\n$2\r\nPx\r\n$3\r\n100\r\n";
        copy_to_array_until(
            &mut fake_app_context_set.request.buffer,
            request_buffer_set,
            0,
            |_, _, source_idx| source_idx == request_buffer_set.len() - 1,
        );
        fake_app_context_set.request.byte_count = request_buffer_set.len();

        parse_resp_proc_command(&mut fake_app_context_set)?;
        assert_eq!(
            fake_app_context_set
                .get_request_resp_command_ref()
                .unwrap()
                .name,
            RespCommandNames::SET
        );
        assert_eq!(
            fake_app_context_set
                .get_request_resp_command_ref()
                .unwrap()
                .parameters
                .len(),
            2
        );

        handle_command_set_async(&mut fake_app_context_set).await?;
        handle_command(&mut fake_app_context_set).await?;
        assert_eq!(
            fake_app_context_set
                .response
                .first()
                .unwrap()
                .command_response
                .to_owned(),
            "+OK\r\n".to_owned()
        );
        assert_eq!(
            fake_app_context_set
                .response
                .first()
                .unwrap()
                .command_response,
            "+OK\r\n".to_owned()
        );

        // Get:
        let mut fake_app_context_get = ConnectionContext::new(&fake_mem_db, &mut fake_tcp_stream)?;
        let request_buffer_get = b"*2\r\n$3\r\ngET\r\n$3\r\nfoo\r\n";
        copy_to_array_until(
            &mut fake_app_context_get.request.buffer,
            request_buffer_get,
            0,
            |_, _, source_idx| source_idx == request_buffer_get.len() - 1,
        );
        fake_app_context_get.request.byte_count = request_buffer_get.len();

        parse_resp_proc_command(&mut fake_app_context_get)?;
        assert_eq!(
            fake_app_context_get
                .get_request_resp_command_ref()
                .unwrap()
                .name,
            RespCommandNames::GET
        );
        assert_eq!(
            fake_app_context_get
                .get_request_resp_command_ref()
                .unwrap()
                .parameters
                .len(),
            1
        );

        handle_command_get_async(&mut fake_app_context_get).await?;
        handle_command(&mut fake_app_context_get).await?;
        assert_eq!(
            fake_app_context_get
                .response
                .first()
                .unwrap()
                .command_response
                .to_owned(),
            "$19\r\nHey world, I'm Joe!\r\n".to_owned()
        );
        assert_eq!(
            fake_app_context_get
                .response
                .first()
                .unwrap()
                .command_response,
            "$19\r\nHey world, I'm Joe!\r\n".to_owned()
        );

        Ok(())
    }

    // #[tokio::test]
    // async fn handle_command_handles_info() -> Result<(), anyhow::Error> {
    //     todo!()
    // }
}
