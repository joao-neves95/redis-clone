use crate::{
    models::cli::{AppCliArgs, AppCliFlagName, CliArgsReplication},
    DEFAULT_LISTENING_PORT,
};

use anyhow::{Error, Result};

pub fn parse_cli_args() -> Result<AppCliArgs, Error> {
    let mut flags = AppCliArgs {
        port: DEFAULT_LISTENING_PORT,
        replica_of: None,
    };

    let mut arg_iter = std::env::args().peekable();
    let mut current_arg = arg_iter.next();

    loop {
        if current_arg.is_none() {
            break;
        }

        let next_arg = arg_iter.peek();

        match &mut current_arg.unwrap().to_lowercase().as_str() {
            &mut AppCliFlagName::PORT | &mut AppCliFlagName::PORT_SHORT => {
                flags.port = match next_arg.unwrap_or(&"".to_owned()).parse::<u32>() {
                    Err(_) => {
                        return Err(Error::msg(
                            "The CLI could not parse port to listen to - Invalid number.",
                        ))
                    }
                    Ok(port) => {
                        arg_iter.next();
                        port
                    }
                }
            }
            &mut AppCliFlagName::REPLICA_OF => {
                // TODO: Actually validate the IP format.
                if next_arg.is_none() {
                    return Err(Error::msg(
                        "The CLI could not parse host for replication - No argument found.",
                    ));
                }

                let next_arg = arg_iter.next().unwrap();

                flags.replica_of = Some(CliArgsReplication {
                    master_host: next_arg,
                    master_port: match arg_iter.next() {
                        None => return Err(Error::msg(
                            "The CLI could not parse port for replication - No argument found.",
                        )),
                        Some(port) => {
                            match port.parse::<u32>() {
                                Err(_) => return Err(Error::msg(
                                    "The CLI could not parse port for replication - Not a valid number.",
                                )),
                                Ok(port) => port
                            }
                        }
                    },
                });
            }

            _ => {}
        }

        current_arg = arg_iter.next();
    }

    Ok(flags)
}
