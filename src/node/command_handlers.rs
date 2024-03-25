use crate::{
    models::{
        connection_context::{ConnectionContext, Handshake, Response},
        db::{
            app_data::AppDataSlave, in_memory_db::EMPTY_RDB_HEX_FILE,
            in_memory_record::InMemoryRecord,
        },
    },
    resp_parser::shared::{RespCommandNames, RespCommandReplConfOption, RespCommandSetOptions},
    utils::{hex_to_utf8_bytes, return_err},
};

use anyhow::{Error, Ok};

pub(crate) fn handle_command_ping<'a>(context: &mut ConnectionContext<'_>) -> Result<(), Error> {
    context.set_response(Response::new_string(format_simple_string(&"PONG")));

    Ok(())
}

pub(crate) async fn handle_command_replconf<'a>(
    context: &mut ConnectionContext<'a>,
) -> Result<(), Error> {
    let mut db_lock = context.mem_db.lock().await;
    let app_data_master = db_lock.get_app_data_mut().get_master_data_mut().unwrap();

    let parameters = &context.get_request_resp_command_ref().unwrap().parameters;

    match parameters.first().unwrap().as_str() {
        RespCommandReplConfOption::LISTENING_PORT => {
            let port = match parameters[1].parse::<u16>() {
                Err(_) => {
                    return return_err(format!(
                        "{} parameter value malformed - Not a number.",
                        RespCommandReplConfOption::LISTENING_PORT
                    ))
                }

                Result::Ok(port) => port,
            };

            context.request.handshake = Handshake::Replica { port };

            app_data_master.slaves.insert(
                port,
                AppDataSlave {
                    port,
                    tcp_stream: context.request.tcp_stream.clone(),
                    full_handshake: false,
                },
            );
        }
        _ => {}
    }

    context.set_response(Response::new_string(format_string_ok()));

    Ok(())
}

/// E.g. input: *3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n
pub(crate) async fn handle_command_psync<'a>(
    context: &mut ConnectionContext<'a>,
) -> Result<(), Error> {
    fn error(port: &u16) -> Result<(), Error> {
        return_err::<()>(format!(
            "Cannot {}, not a replica, or command {} no yet performed for port {}.",
            RespCommandNames::PSYNC,
            RespCommandNames::REPLCONF,
            port,
        ))
    }

    let mut db_lock = context.mem_db.lock().await;
    let app_data_master = db_lock.get_app_data_mut().get_master_data_mut().unwrap();

    let port = match context.request.handshake {
        Handshake::None => return error(&0),
        Handshake::Replica { port } => port,
    };

    let slave = app_data_master.slaves.get_mut(&port);

    if slave.is_none() {
        return error(&port);
    }

    let slave = slave.unwrap();
    slave.full_handshake = true;

    let response = format_simple_string(&format!(
        "FULLRESYNC {} {}",
        app_data_master.replid, app_data_master.repl_offset
    ));

    let mut decoded_rdb_file = hex_to_utf8_bytes(EMPTY_RDB_HEX_FILE)?;

    let mut rdb_file_response = format!("${}\r\n", decoded_rdb_file.len())
        .as_bytes()
        .to_vec();
    rdb_file_response.append(&mut decoded_rdb_file);

    context.add_response(Response::new_string(response));
    context.add_response(Response::new_byte(rdb_file_response));

    Ok(())
}

pub(crate) fn handle_command_echo<'a>(context: &mut ConnectionContext<'_>) -> Result<(), Error> {
    let message = context
        .get_request_resp_command_ref()
        .unwrap()
        .parameters
        .first()
        .unwrap();

    context.set_response(Response::new_string(format_bulk_string(message)));

    Ok(())
}

pub(crate) async fn handle_command_info<'a>(
    context: &mut ConnectionContext<'a>,
) -> Result<(), Error> {
    let db_lock = context.mem_db.lock().await;
    let app_data = db_lock.get_app_data_ref();

    context.set_response(Response {
        command_response: format_bulk_string(&format!(
            "# Replication\r\nrole:{}\r\nconnected_slaves:0{}",
            if app_data.get_replication_data_ref().is_none() {
                "master"
            } else {
                "slave"
            },
            if app_data.get_replication_data_ref().is_some() {
                "".to_owned()
            } else {
                let master_data = app_data.get_master_data_ref().unwrap();
                format!(
                    "\r\nmaster_replid:{}\r\nmaster_repl_offset:{}",
                    master_data.replid, master_data.repl_offset
                )
            }
        )),
        command_byte_response: None,
    });

    Ok(())
}

/// Example commands:
/// "redis-cli set foo bar"
/// "redis-cli set foo bar px 100" (px = key expiry)
pub(crate) async fn handle_command_set_async<'a>(
    context: &mut ConnectionContext<'a>,
) -> Result<(), Error> {
    let mut db_lock = context.mem_db.lock().await;
    let parameters = &context.get_request_resp_command_ref().unwrap().parameters;

    let expiry =
        if parameters.len() == 2 {
            None
        } else if parameters[2].to_uppercase() == RespCommandSetOptions::EXPIRY {
            match parameters[3].parse::<u128>() {
                Err(_) => return Err(Error::msg(
                    "Could not parse command: The SET command's PX option only accepts numbers.",
                )),
                Result::Ok(expiry) => Some(expiry),
            }
        } else {
            return Err(Error::msg(
                "Could not parse command: The SET command correctly only supports the PX option.",
            ));
        };

    (*db_lock).get_records_ref_mut().insert(
        parameters[0].to_owned(),
        InMemoryRecord::new(parameters[1].to_owned(), expiry),
    );

    context.set_response(Response::new_string(format_string_ok()));

    Ok(())
}

pub(crate) async fn handle_command_get_async<'a>(
    context: &mut ConnectionContext<'a>,
) -> Result<(), Error> {
    let key = &context.get_request_resp_command_ref().unwrap().parameters[0];

    let mut db_lock = context.mem_db.lock().await;

    let existing_value = (*db_lock)
        .get_records_ref_mut()
        .get(key)
        .map(|val| val.to_owned());

    context.set_response(Response {
        command_response: if existing_value.is_none() {
            format_null_bulk_string()
        } else {
            let existing_value = existing_value.unwrap();

            if existing_value.has_expired()? {
                (*db_lock).get_records_ref_mut().remove(key);

                format_null_bulk_string()
            } else {
                format_bulk_string(&existing_value.value.to_owned())
            }
        },
        command_byte_response: None,
    });

    Ok(())
}

fn format_null_bulk_string() -> String {
    "$-1\r\n".to_owned()
}

fn format_string_ok() -> String {
    format_simple_string(&"OK")
}

fn format_simple_string(message: &str) -> String {
    format!("+{}\r\n", message)
}

fn format_bulk_string(message: &str) -> String {
    format!("${}\r\n{}\r\n", message.len(), message)
}
