use super::{
    db::{app_data::AppData, in_memory_db::InMemoryDb},
    t_stream::TStream,
};
use crate::{resp_parser::shared::RespCommand, TCP_RESPONSE_BUFFER_SIZE};

use std::{fmt::Debug, sync::Arc};

use anyhow::Error;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct ConnectionContext<'a> {
    pub mem_db: &'a Arc<Mutex<InMemoryDb>>,
    pub request: Request<'a>,

    /// Each response value is written separably into the TCP stream.
    pub response: Vec<Response>,
}

impl<'a> ConnectionContext<'a> {
    pub fn new(
        mem_db: &'a Arc<Mutex<InMemoryDb>>,
        tcp_stream: &'a Arc<Mutex<dyn TStream>>,
    ) -> Result<Self, Error> {
        Ok(ConnectionContext {
            mem_db,
            request: Request::new(tcp_stream),
            response: Vec::<Response>::new(),
        })
    }

    pub fn reset(&mut self) -> &Self {
        self.request.buffer = [0; TCP_RESPONSE_BUFFER_SIZE];
        self.request.byte_count = 0;
        self.request.resp_command = None;
        self.response = Vec::new();

        self
    }

    pub async fn println_by(&self, message: &str) {
        let db_lock = self.mem_db.lock().await;

        println!(
            "{}({}) -> {message}",
            if db_lock.get_app_data_ref().get_master_data_ref().is_some() {
                "master"
            } else {
                "slave"
            },
            db_lock.get_app_data_ref().listening_port
        )
    }

    pub fn get_request_ref(&self) -> &Request {
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

    pub fn set_response(&mut self, command_response: Response) -> &Self {
        self.response = vec![command_response];

        self
    }

    pub fn add_response(&mut self, command_response: Response) -> &Self {
        self.response.push(command_response);

        self
    }

    pub fn format_request_info(&self, include_mem_db: bool) -> Result<String, Error> {
        let empty_mem_db = InMemoryDb::new(AppData::new_master(0)?)?;

        Ok(format!(
            "request: {:?},\nmem_db: {:?}",
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
pub struct InternalRequest {
    pub buffer: [u8; TCP_RESPONSE_BUFFER_SIZE],
    pub byte_count: usize,
}

#[derive(Debug)]
pub struct Request<'a> {
    pub buffer: [u8; TCP_RESPONSE_BUFFER_SIZE],
    pub byte_count: usize,
    pub resp_command: Option<RespCommand>,
    pub tcp_stream: &'a Arc<Mutex<dyn TStream>>,
    pub handshake: Handshake,
}

#[derive(Debug)]
pub enum Handshake {
    None,
    Replica { port: u16 },
}

impl<'a> Request<'a> {
    pub fn new(tcp_stream: &'a Arc<Mutex<dyn TStream>>) -> Self {
        Request {
            buffer: [0; TCP_RESPONSE_BUFFER_SIZE],
            byte_count: 0,
            resp_command: None,
            tcp_stream,
            handshake: Handshake::None,
        }
    }
}

#[derive(Debug)]
pub struct Response {
    pub command_response: String,
    pub command_byte_response: Option<Vec<u8>>,
}

impl Response {
    pub fn new_string(response: String) -> Self {
        Self {
            command_response: response,
            command_byte_response: None,
        }
    }

    pub fn new_byte(response: Vec<u8>) -> Self {
        Self {
            command_response: "".to_string(),
            command_byte_response: Some(response),
        }
    }
}

impl Clone for Response {
    fn clone(&self) -> Self {
        Self {
            command_response: self.command_response.clone(),
            command_byte_response: self.command_byte_response.clone(),
        }
    }
}
