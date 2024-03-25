use crate::{models::connection_context::ConnectionContext, resp_parser::shared::RespCommandType};

use anyhow::Error;
use tokio::io::AsyncWriteExt;

pub(crate) async fn propagate<'a>(
    connection_context: &mut ConnectionContext<'a>,
) -> Result<(), Error> {
    if connection_context
        .request
        .resp_command
        .as_ref()
        .unwrap()
        .command_type
        != RespCommandType::Write
    {
        return Ok(());
    }

    let db_lock = connection_context.mem_db.lock().await;
    let app_data = db_lock.get_app_data_ref();

    if app_data.get_master_data_ref().is_none() {
        return Ok(());
    }

    println!("propagating command to all slaves...");

    // To remove the null/0 bytes at the end of the original buffer.
    let original_request =
        &connection_context.request.buffer[0..connection_context.request.byte_count];

    for (_, slave) in &app_data.get_master_data_ref().unwrap().slaves {
        if !slave.full_handshake {
            continue;
        }

        println!("slave port: {}", slave.port);

        let mut slave_tcp_lock = slave.tcp_stream.lock().await;
        slave_tcp_lock.write_all(&original_request).await?;
        slave_tcp_lock.flush().await?;
    }

    println!("finished propagating.");
    Ok(())
}
