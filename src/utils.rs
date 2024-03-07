use std::time::SystemTime;

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

    pub const LF_CHAR: char = '\n';
    pub const LF_BYTE: u8 = b'\n';

    pub const CR_CHAR: char = '\r';
    pub const CR_BYTE: u8 = b'\r';
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

const MAX_ASCII_CHAR: u8 = 127;

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

    use super::pseudo_random_number;
    use crate::utils::{pseudo_random_ascii, pseudo_random_ascii_alphanumeric, u32_count};

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
