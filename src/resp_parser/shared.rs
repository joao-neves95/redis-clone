pub struct RespCommandNames {}

impl RespCommandNames {
    pub const PING: &'static str = "PING";
    pub const REPLCONF: &'static str = "REPLCONF";
    pub const PSYNC: &'static str = "PSYNC";
    pub const ECHO: &'static str = "ECHO";
    pub const INFO: &'static str = "INFO";
    pub const GET: &'static str = "GET";
    pub const SET: &'static str = "SET";
}

pub struct RespCommandResponseNames {}

impl RespCommandResponseNames {
    pub const OK: &'static str = "OK";
    pub const PONG: &'static str = "PONG";
}

pub struct RespCommandSetOptions {}

impl RespCommandSetOptions {
    pub const EXPIRY: &'static str = "PX";
}

pub struct RespDataTypesFirstByte {}

impl RespDataTypesFirstByte {
    pub const ARRAYS_STR: &'static str = "*";

    pub const BULK_STRINGS_CHAR: char = '$';

    pub const SIMPLE_STRINGS_CHAR: char = '+';
    pub const SIMPLE_STRINGS_BYTE: u8 = b'+';
}

#[derive(Debug)]
pub enum RespDataType {
    BulkString { size: u32, value: String },
    SimpleString { value: String },
}

impl RespDataType {
    pub fn get_value_string(&self) -> String {
        match self {
            RespDataType::BulkString { size: _, value } => value.to_owned(),
            RespDataType::SimpleString { value } => value.to_owned(),
        }
    }
}

#[derive(Debug)]
pub struct RespCommand {
    pub name: String,

    pub parameters: Vec<String>,
}
