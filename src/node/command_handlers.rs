use crate::{
    models::{
        app_context::{AppContext, Response},
        db::InMemoryRecord,
    },
    resp_parser::shared::RespCommandSetOptions,
};

use anyhow::{Error, Ok};

pub(crate) fn handle_command_ping<'a>(context: &mut AppContext<'_>) -> Result<(), Error> {
    context.response = Some(Response {
        command_response: format_simple_string(&"PONG"),
    });

    Ok(())
}

pub(crate) fn handle_command_replconf<'a>(context: &mut AppContext<'_>) -> Result<(), Error> {
    context.response = Some(Response {
        command_response: format_string_ok(),
    });

    Ok(())
}

/// E.g. input: *3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n
pub(crate) async fn handle_command_psync<'a>(context: &mut AppContext<'_>) -> Result<(), Error> {
    let mut db_lock = context.get_mem_db_ref().lock().await;
    let app_data_master = db_lock.get_app_data_ref_mut().master.as_mut().unwrap();

    context.response = Some(Response {
        command_response: format_simple_string(&format!(
            "FULLRESYNC {} {}",
            app_data_master.replid, app_data_master.repl_offset
        )),
    });

    Ok(())
}

pub(crate) fn handle_command_echo<'a>(context: &mut AppContext<'_>) -> Result<(), Error> {
    let message = context
        .get_request_resp_command_ref()
        .unwrap()
        .parameters
        .first()
        .unwrap();

    context.response = Some(Response {
        command_response: format_bulk_string(message),
    });

    Ok(())
}

pub(crate) async fn handle_command_info<'a>(context: &mut AppContext<'_>) -> Result<(), Error> {
    let db_lock = context.get_mem_db_ref().lock().await;
    let app_data = db_lock.get_app_data_ref();

    context.set_response_command_response(format_bulk_string(&format!(
        "# Replication\r\nrole:{}\r\nconnected_slaves:0{}",
        if app_data.replica.is_none() {
            "master"
        } else {
            "slave"
        },
        if app_data.replica.is_some() {
            "".to_owned()
        } else {
            let master_data = app_data.master.as_ref().unwrap();
            format!(
                "\r\nmaster_replid:{}\r\nmaster_repl_offset:{}",
                master_data.replid, master_data.repl_offset
            )
        }
    )));

    Ok(())
}

/// Example commands:
/// "redis-cli set foo bar"
/// "redis-cli set foo bar px 100" (px = key expiry)
pub(crate) async fn handle_command_set_async<'a>(
    context: &mut AppContext<'_>,
) -> Result<(), Error> {
    let mut db_lock = context.get_mem_db_ref().lock().await;
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

    context.set_response_command_response(format_string_ok());

    Ok(())
}

pub(crate) async fn handle_command_get_async<'a>(
    context: &mut AppContext<'_>,
) -> Result<(), Error> {
    let key = &context.get_request_resp_command_ref().unwrap().parameters[0];

    let mut db_lock = context.get_mem_db_ref().lock().await;

    let existing_value = (*db_lock)
        .get_records_ref_mut()
        .get(key)
        .map(|val| val.to_owned());

    context.set_response_command_response(if existing_value.is_none() {
        format_null_bulk_string()
    } else {
        let existing_value = existing_value.unwrap();

        if existing_value.has_expired()? {
            (*db_lock).get_records_ref_mut().remove(key);

            format_null_bulk_string()
        } else {
            format_bulk_string(&existing_value.value.to_owned())
        }
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
