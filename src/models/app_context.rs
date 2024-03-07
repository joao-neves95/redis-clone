use crate::{resp_parser::shared::RespCommand, utils::delete_bytes_after_first_crlf};

use super::db::{AppData, InMemoryDb};

use std::sync::Arc;

use anyhow::Error;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct AppContext<'a> {
    mem_db: &'a Arc<Mutex<InMemoryDb>>,

    request: Request<'a>,

    pub response: Option<Response>,
}

impl<'a> AppContext<'a> {
    pub fn new(
        mem_db: &'a Arc<Mutex<InMemoryDb>>,
        raw_request_buffer: &'a [u8],
    ) -> Result<Self, Error> {
        Ok(AppContext {
            mem_db,
            request: Request::new(&raw_request_buffer)?,
            response: None,
        })
    }

    pub fn new_request(
        mem_db: &'a Arc<Mutex<InMemoryDb>>,
        request: Request<'a>,
    ) -> Result<Self, Error> {
        Ok(AppContext {
            mem_db,
            request,
            response: None,
        })
    }

    pub fn get_mem_db_ref(&self) -> &'a Arc<Mutex<InMemoryDb>> {
        &self.mem_db
    }

    pub fn get_request_ref(&self) -> &Request<'a> {
        &self.request
    }

    pub fn set_request_resp_command(&mut self, resp_command: RespCommand) -> &Self {
        if self.request.resp_command.is_none() {
            self.request.resp_command = Some(resp_command);
        }

        self
    }

    pub fn get_request_resp_command_ref(&self) -> Option<&RespCommand> {
        self.request.resp_command.as_ref()
    }

    pub fn set_response_command_response(&mut self, command_response: String) -> &Self {
        self.response = Some(Response { command_response });

        self
    }

    pub fn unwrap_response_command_response(&self) -> &String {
        &self.response.as_ref().unwrap().command_response
    }

    pub fn format_request_info(&self, include_mem_db: bool) -> Result<String, Error> {
        let empty_mem_db = InMemoryDb::new(AppData::new_master(0)?)?;

        Ok(format!(
            "request: {:?}, mem_db: {:?}",
            self.get_request_resp_command_ref().unwrap(),
            if include_mem_db {
                self.mem_db
            } else {
                &empty_mem_db
            }
        ))
    }
}

#[derive(Debug)]
pub struct Request<'a> {
    pub raw_command: &'a str,
    // pub raw_command: String,
    pub resp_command: Option<RespCommand>,
}

impl<'a> Request<'a> {
    pub fn new(raw_command_bytes: &'a [u8]) -> Result<Self, Error> {
        Ok(Request {
            raw_command: std::str::from_utf8(raw_command_bytes)?,
            resp_command: None,
        })
    }

    pub fn new_simple_string(raw_command_bytes: &'a mut [u8]) -> Result<Self, Error> {
        Ok(Request {
            raw_command: std::str::from_utf8(delete_bytes_after_first_crlf(raw_command_bytes))?,
            resp_command: None,
        })
    }
}

#[derive(Debug)]
pub struct Response {
    pub command_response: String,
}
