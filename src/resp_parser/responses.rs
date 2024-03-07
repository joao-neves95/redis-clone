use anyhow::Error;

use crate::models::app_context::AppContext;

use super::{
    data_types::{move_resp_bulk_string, move_resp_simple_string},
    shared::{RespDataType, RespDataTypesFirstByte},
};

pub(crate) fn parse_redis_resp_proc_response(
    context: &mut AppContext<'_>,
) -> Result<RespDataType, Error> {
    let raw_command = context.get_request_ref().raw_command;

    if raw_command.is_empty() {
        return Err(Error::msg("Could not parse response: Command is empty."));
    }

    let mut command_body_iter = raw_command.chars().enumerate().peekable();
    let current_char: Option<(usize, char)> = command_body_iter.next();

    Ok(match current_char.unwrap().1 {
        RespDataTypesFirstByte::BULK_STRINGS_CHAR => {
            move_resp_bulk_string(&mut command_body_iter, &current_char)?
        }
        RespDataTypesFirstByte::SIMPLE_STRINGS_CHAR => {
            move_resp_simple_string(&mut command_body_iter, &current_char)?
        }

        _ => {
            return Err(Error::msg(
                "Could not parse response: Unknown or not implemented data type.",
            ))
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        models::app_context::AppContext,
        resp_parser::{parse_redis_resp_proc_response, shared::RespCommandResponseNames},
        test_helpers::utils::create_test_mem_db,
    };

    #[tokio::test]
    async fn parse_redis_resp_proc_response_should_parse_simple_strings(
    ) -> Result<(), anyhow::Error> {
        let fake_mem_db = create_test_mem_db()?;

        let request_buffer = b"+OK\r\n";
        let mut fake_app_context = AppContext::new(&fake_mem_db, request_buffer)?;

        assert_eq!(
            parse_redis_resp_proc_response(&mut fake_app_context)?.get_value_string(),
            RespCommandResponseNames::OK
        );

        let request_buffer = b"+PONG\r\n";
        let mut fake_app_context = AppContext::new(&fake_mem_db, request_buffer)?;

        assert_eq!(
            parse_redis_resp_proc_response(&mut fake_app_context)?.get_value_string(),
            RespCommandResponseNames::PONG
        );

        Ok(())
    }
}
