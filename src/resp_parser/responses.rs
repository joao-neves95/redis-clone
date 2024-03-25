use super::{
    data_types::{move_resp_bulk_string, move_resp_simple_string},
    shared::{RespDataType, RespDataTypesFirstByte},
};
use crate::models::connection_context::InternalRequest;

use anyhow::Error;

pub(crate) fn parse_redis_resp_proc_response(
    request: &InternalRequest,
) -> Result<RespDataType, Error> {
    if request.buffer.starts_with(&[0]) {
        return Err(Error::msg("Could not parse response: Command is empty."));
    }

    // TODO: Refactor this method to use the raw byte themselves.
    let mut command_body_iter = std::str::from_utf8(&request.buffer)?
        .chars()
        .enumerate()
        .peekable();
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
        models::connection_context::InternalRequest,
        resp_parser::{parse_redis_resp_proc_response, shared::RespCommandResponseNames},
        utils::copy_to_array_until,
    };

    #[tokio::test]
    async fn parse_redis_resp_proc_response_should_parse_simple_strings(
    ) -> Result<(), anyhow::Error> {
        let mut request_buffer = [0; 1024];
        copy_to_array_until(&mut request_buffer, b"+OK\r\n\0", 0, |byte, _, _| byte == 0);

        assert_eq!(
            parse_redis_resp_proc_response(&InternalRequest {
                buffer: request_buffer
            })?
            .get_value_string(),
            RespCommandResponseNames::OK
        );

        let mut request_buffer = [0; 1024];
        copy_to_array_until(&mut request_buffer, b"+PONG\r\n\0", 0, |byte, _, _| {
            byte == 0
        });

        assert_eq!(
            parse_redis_resp_proc_response(&InternalRequest {
                buffer: request_buffer
            })?
            .get_value_string(),
            RespCommandResponseNames::PONG
        );

        Ok(())
    }
}
