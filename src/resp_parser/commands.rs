use anyhow::{Error, Result};

use crate::{
    models::app_context::AppContext, resp_parser::data_types::move_resp_bulk_string,
    utils::LineEndings,
};

use super::{
    data_types::move_to_crlf_end,
    shared::{RespCommand, RespDataTypesFirstByte},
};

/// Input examples: <br/>
/// "*1\r\n$4\r\nping\r\n" <br/>
/// "*2\r\n$4\r\necho\r\n$3\r\nhey\r\n" <br/>
pub(crate) fn parse_resp_proc_command(context: &mut AppContext<'_>) -> Result<(), Error> {
    let raw_command = context.get_request_ref().raw_command;

    if !raw_command.starts_with(RespDataTypesFirstByte::ARRAYS_STR) {
        return Err(Error::msg(
            "Could not parse response: Command malformed - not an array.",
        ));
    }

    let raw_command = raw_command.split_once(LineEndings::CRLF_STR);

    if raw_command.is_none() {
        return Err(Error::msg(
            "Could not parse command: Command array malformed.",
        ));
    }

    let raw_command = raw_command.unwrap();

    if raw_command.0.len() < 2 {
        return Err(Error::msg("Could not parse command: Command malformed."));
    }

    let (array_type, num_of_parts) = raw_command.0.split_at(1);

    if array_type != RespDataTypesFirstByte::ARRAYS_STR {
        return Err(Error::msg(
            "Could not parse command: Command malformed, the command is not an array.",
        ));
    }

    let mut command_body_iter = raw_command.1.chars().enumerate().peekable();
    let current_char: Option<(usize, char)> = command_body_iter.next();

    let command_name = move_resp_bulk_string(&mut command_body_iter, &current_char)?;
    let command_name = command_name.get_value_string().to_ascii_uppercase();

    if command_name.is_empty() {
        return Err(Error::msg("Could not parse command: Command is empty."));
    }

    let current_char: Option<(usize, char)> = command_body_iter.next();

    context.set_request_resp_command(parse_resp_multi_param_command_body(
        &command_name,
        num_of_parts.parse::<u8>()? - 1,
        &mut command_body_iter,
        &current_char,
    )?);

    Ok(())
}

fn parse_resp_multi_param_command_body<'a>(
    command_name: &'a String,
    parameter_count: u8,
    command_iter: &mut std::iter::Peekable<std::iter::Enumerate<std::str::Chars<'_>>>,
    current_char: &Option<(usize, char)>,
) -> Result<RespCommand, Error> {
    let mut curr_char = *current_char;
    let mut parameters = Vec::<String>::new();

    for _ in 0..parameter_count {
        let param = move_resp_bulk_string(command_iter, &curr_char)?
            .get_value_string()
            .to_owned();

        parameters.push(param);
        curr_char = command_iter.next();
    }

    move_to_crlf_end(command_iter);

    Ok(RespCommand {
        name: command_name.to_owned(),
        parameters,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        models::app_context::AppContext,
        resp_parser::{parse_resp_proc_command, shared::RespCommandNames},
        test_helpers::utils::create_test_mem_db,
    };

    #[tokio::test]
    async fn parse_resp_proc_command_should_parse_known_commands() -> Result<(), anyhow::Error> {
        let request_buffer = b"*1\r\n$4\r\npiNg\r\n";

        let fake_mem_db = create_test_mem_db()?;
        let mut fake_app_context = AppContext::new(&fake_mem_db, request_buffer)?;

        parse_resp_proc_command(&mut fake_app_context)?;
        assert_eq!(
            fake_app_context
                .get_request_resp_command_ref()
                .unwrap()
                .name,
            RespCommandNames::PING
        );

        Ok(())
    }
}
