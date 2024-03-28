use std::{iter::Map, ops::Index, time::SystemTime};

use anyhow::Error;

const PRIME_NUMBER: u128 = U8number::FIVE as u128;

pub struct U8number {}

impl U8number {
    pub const FIVE: u8 = 5;
}

pub struct U32number {}

impl U32number {
    pub const ZERO: u32 = 0;
}

pub struct U128number {}

impl U128number {
    pub const ZERO: u128 = 0;
}

pub struct LineEndings {}

impl LineEndings {
    pub const CRLF_STR: &'static str = "\r\n";
    pub const CRLF_BYTES: &'static [u8] = b"\r\n";

    pub const LF_CHAR: char = '\n';
    pub const LF_BYTE: u8 = b'\n';

    pub const CR_CHAR: char = '\r';
    pub const CR_BYTE: u8 = b'\r';
}

pub fn return_err<T>(message: String) -> Result<T, Error> {
    return Err(Error::msg(message));
}

pub fn hex_to_utf8_bytes(hex_buff: &[u8]) -> Result<Vec<u8>, Error> {
    let bytes = hex_buff
        .chunks(2)
        .map(|hex_byte_chunk| {
            let hex_str = match std::str::from_utf8(hex_byte_chunk) {
                Err(_) => panic!("Could not parse hex buffer: Not UTF-8."),
                Ok(str) => str,
            };

            match u8::from_str_radix(hex_str, 16) {
                Err(_) => panic!("Could not parse hex buffer: Not HEX."),
                Ok(hex_byte) => hex_byte,
            }
        })
        .collect();

    Ok(bytes)
}

/// Splits the string on the first occurrence of the specified delimiter and
/// returns prefix before delimiter and suffix after delimiter.
///
/// # Example:
/// ```rust
/// let source_bytes: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9][..];
/// let delimiter: &[u8] = &[3, 4][..];
///
/// assert_eq!(
///     split_slice_once(source_bytes, delimiter),
///     Some((&[1, 2][..], &[5, 6, 7, 8, 9][..]))
/// );
/// ```
pub fn split_u8_slice_once<'a>(
    source: &'a [u8],
    delimiter: &'a [u8],
) -> Option<(&'a [u8], &'a [u8])> {
    match find_first_index_in_u8_slice(source, delimiter) {
        None => None,
        Some(first_idx) => Some((
            &source[0..first_idx],
            &source[first_idx + delimiter.len()..],
        )),
    }
}

/// If not found it returns `source.len()`.
pub fn find_first_index_in_u8_slice(source: &[u8], query: &[u8]) -> Option<usize> {
    for i in 0..source.len() {
        if &source[i..i + query.len()] == query {
            return Some(i);
        }
    }

    None
}

pub fn u8_slice_into_char_slice(source: &[u8], target: &mut [char]) {
    for i in 0..source.len() {
        target[i] = source[i] as char;
    }
}

/// It will stop when it reaches a 0 value byte. At the moment this has no overflow protection whatsoever.
pub fn delete_bytes_after_first_crlf(buff: &mut [u8]) -> &[u8] {
    let mut i = 0;
    let mut delete = false;

    loop {
        if buff[i] == 0 {
            return buff;
        }

        if delete {
            buff[i] = 0;
            i = i + 1;
            continue;
        }

        if buff[i] != LineEndings::CR_BYTE {
            i = i + 1;
            continue;
        }

        if buff[i + 1] == LineEndings::LF_BYTE {
            delete = true;
            i = i + 1;
            continue;
        }
    }
}

pub fn concat_u32(left: u32, right: u32) -> Option<u32> {
    let pow_result = 10u32.checked_pow(u32_count(right));

    if pow_result.is_none() {
        return None;
    }

    Some((left * pow_result.unwrap()) + right)
}

pub fn u32_count(value: u32) -> u32 {
    let mut counter = 1;
    let mut width_meter = 10;

    while width_meter <= value {
        width_meter *= 10;
        counter += 1;
    }

    counter
}

/// Copies `source` from index 0 into `target` from the inclusive `start` and stops when the result of the `until` closure is true. <br/>
/// This function does not offer overflow protection, so the caller has to account for that.
///
/// `start`: start index of target to copy into.
/// `until`: closure with a predicate to stop the copy. Receives (current item, current target index, current source index).
pub fn copy_to_array_until<T, F>(target: &mut [T], source: &[T], start: usize, until: F)
where
    T: Copy,
    F: Fn(T, usize, usize) -> bool,
{
    let mut i_target: usize = start;
    let mut i_source: usize = 0;

    loop {
        let item = source[i_source];

        if until(item, i_target, i_source) {
            break;
        }

        target[i_target] = item;
        i_target += 1;
        i_source += 1;
    }
}

// const ALPHANUMERIC_BYTES: &[u8; 62] = b"1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ALPHANUMERIC_BYTES: &[u8; 62] =
    b"1a2b3c4d5e6f7g8h9i0jklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub fn pseudo_random_ascii_alphanumeric(size: u32) -> Result<String, Error> {
    let mut value = String::new();

    for i in 0..size {
        let random_idx = pseudo_random_min_max_number(
            pseudo_random_min_max_number(1, 1, 2, PRIME_NUMBER as u32)? as u8,
            0,
            61,
            i + PRIME_NUMBER as u32,
        )? as usize;

        value.push(ALPHANUMERIC_BYTES[random_idx].into());
    }

    Ok(value)
}

#[allow(dead_code)]
const MAX_ASCII_CHAR: u8 = 127;

#[allow(dead_code)]
pub fn pseudo_random_ascii(size: u32) -> Result<String, Error> {
    let mut value = String::new();

    for i in 0..size {
        let char_num = pseudo_random_min_max_number(
            pseudo_random_min_max_number(1, 2, 3, PRIME_NUMBER as u32)? as u8,
            33,
            MAX_ASCII_CHAR.into(),
            i + PRIME_NUMBER as u32,
        )?;

        value.push(char_num as u8 as char);
    }

    Ok(value)
}

pub fn pseudo_random_min_max_number(
    size: u8,
    min_num: u32,
    max_num: u32,
    seed: u32,
) -> Result<u32, Error> {
    let mut num = max_num + 1;

    while num < min_num || num > max_num {
        num = pseudo_random_number(size, seed)?;
    }

    Ok(num)
}

/// It will always return a number of length `size`.
/// It overflows if `size` >= 10.
pub fn pseudo_random_number(size: u8, seed: u32) -> Result<u32, Error> {
    if size > 9 {
        return Err(Error::msg(
            "Can only produce random numbers of length 9 or less.",
        ));
    }

    let seed = seed as u128;
    let max_size = size as u32;

    let random_pointer = Box::new(PRIME_NUMBER);
    let random_pointer = std::ptr::addr_of!(random_pointer) as u128;

    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_micros();

    let size_multiplier = 10u32.checked_pow(max_size);

    if size_multiplier.is_none() {
        return Err(Error::msg("Overflow"));
    }

    let size_multiplier = size_multiplier.unwrap() as u128;

    let mut num = U32number::ZERO;
    let mut padding = U128number::ZERO;

    while num == U32number::ZERO || u32_count(num) != size as u32 {
        num = (((time + random_pointer + seed + padding) + PRIME_NUMBER) % size_multiplier) as u32;
        padding += 1;
    }

    Ok(num)
}

#[cfg(test)]
mod tests {
    use anyhow::{Error, Result};

    use super::{find_first_index_in_u8_slice, pseudo_random_number};
    use crate::utils::{
        pseudo_random_ascii, pseudo_random_ascii_alphanumeric, split_u8_slice_once, u32_count,
    };

    #[test]
    fn find_first_index_in_slice_passes() {
        let source_bytes = b"first\r\nsecond\r\nthird";
        let query = b"\r\n";

        assert_eq!(find_first_index_in_u8_slice(source_bytes, query), Some(5));
    }

    #[test]
    fn split_slice_once_passes() {
        let source_bytes = b"first\r\nsecond\r\nthird";
        let delimiter = b"\r\n";

        assert_eq!(
            split_u8_slice_once(source_bytes, delimiter),
            Some((b"first" as &[u8], b"second\r\nthird" as &[u8]))
        );

        let source_bytes: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9][..];
        let delimiter: &[u8] = &[3, 4][..];

        assert_eq!(
            split_u8_slice_once(source_bytes, delimiter),
            Some((&[1, 2][..], &[5, 6, 7, 8, 9][..]))
        );
    }

    #[test]
    fn u32_count_passes() {
        assert_eq!(u32_count(0), 1);
        assert_eq!(u32_count(1), 1);
        assert_eq!(u32_count(1234), 4);
        assert_eq!(u32_count(123456789), 9);
    }

    #[test]
    fn pseudo_random_number_passes() -> Result<(), Error> {
        let num = pseudo_random_number(1, 0)?;
        assert_eq!(u32_count(num), 1);

        let num = pseudo_random_number(3, 0)?;
        assert_eq!(u32_count(num), 3);

        let num = pseudo_random_number(9, 0)?;
        assert_eq!(u32_count(num), 9);

        Ok(())
    }

    #[test]
    #[should_panic]
    fn pseudo_random_number_should_panic_if_length_more_10() {
        pseudo_random_number(10, 0).unwrap();
    }

    #[test]
    fn pseudo_random_ascii_passes() -> Result<(), Error> {
        let value = pseudo_random_ascii(1)?;
        assert_eq!(value.chars().count(), 1);
        println!("{}", value);

        let value = pseudo_random_ascii(5)?;
        assert_eq!(value.chars().count(), 5);
        println!("{}", value);

        let value = pseudo_random_ascii(10)?;
        assert_eq!(value.chars().count(), 10);
        println!("{}", value);

        let value = pseudo_random_ascii(40)?;
        assert_eq!(value.chars().count(), 40);
        println!("{}", value);

        Ok(())
    }

    #[test]
    fn pseudo_random_ascii_alphanumeric_passes() -> Result<(), Error> {
        let value = pseudo_random_ascii_alphanumeric(1)?;
        assert_eq!(value.chars().count(), 1);
        println!("{}", value);

        let value = pseudo_random_ascii_alphanumeric(5)?;
        assert_eq!(value.chars().count(), 5);
        println!("{}", value);

        let value = pseudo_random_ascii_alphanumeric(10)?;
        assert_eq!(value.chars().count(), 10);
        println!("{}", value);

        let value = pseudo_random_ascii_alphanumeric(40)?;
        assert_eq!(value.chars().count(), 40);
        println!("{}", value);

        Ok(())
    }
}
