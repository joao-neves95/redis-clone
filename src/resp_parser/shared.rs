pub struct RespDataTypesFirstByte {}

impl RespDataTypesFirstByte {
    pub const ARRAYS_STR: &'static str = "*";
    pub const ARRAYS_BYTE: &'static [u8] = b"*";

    pub const BULK_STRINGS_CHAR: char = '$';

    pub const SIMPLE_STRINGS_CHAR: char = '+';
    pub const SIMPLE_STRINGS_BYTE: u8 = b'+';
}

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

#[derive(Debug, PartialEq)]
pub enum RespCommandType {
    Read,
    Write,
}

impl RespCommandType {
    pub fn from_command_name(command_name: &str) -> RespCommandType {
        match command_name {
            RespCommandNames::SET => RespCommandType::Write,
            _ => RespCommandType::Read,
        }
    }
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

pub struct RespCommandReplConfOption {}

impl RespCommandReplConfOption {
    pub const LISTENING_PORT: &'static str = "listening-port";
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

    pub command_type: RespCommandType,

    pub parameters: Vec<String>,
}
