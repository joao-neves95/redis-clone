use super::{
    data_types::{move_resp_bulk_string, move_resp_simple_string},
    shared::{RespDataType, RespDataTypesFirstByte},
};
use crate::{
    models::connection_context::InternalRequest, utils::u8_slice_into_char_slice,
    TCP_RESPONSE_BUFFER_SIZE,
};

use anyhow::Error;

pub(crate) fn parse_redis_resp_proc_response(
    request: &InternalRequest,
) -> Result<RespDataType, Error> {
    if request.buffer.starts_with(&[0]) {
        return Err(Error::msg("Could not parse response: Command is empty."));
    }

    let mut raw_command_chars = [0 as char; TCP_RESPONSE_BUFFER_SIZE];
    u8_slice_into_char_slice(
        &request.buffer[0..request.byte_count],
        &mut raw_command_chars,
    );
    let raw_command_chars = raw_command_chars;

    // TODO: Refactor this to use the raw byte slice instead of the iterator.
    let mut command_body_iter = raw_command_chars[..request.byte_count]
        .iter()
        .enumerate()
        .peekable();

    let current_char: Option<(usize, &char)> = command_body_iter.next();

    Ok(match current_char.unwrap().1 {
        &RespDataTypesFirstByte::BULK_STRINGS_CHAR => {
            move_resp_bulk_string(&mut command_body_iter, &current_char)?
        }
        &RespDataTypesFirstByte::SIMPLE_STRINGS_CHAR => {
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
                buffer: request_buffer,
                byte_count: 50,
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
                buffer: request_buffer,
                byte_count: 50,
            })?
            .get_value_string(),
            RespCommandResponseNames::PONG
        );

        Ok(())
    }
}
