# Redis Clone RS

A Redis clone made in vanilla Rust, and Tokio for multithreading.

## Architecture
- Command listener:
    - The server listens for connections and handles them in [./src/resp_parser/commands.rs](./src/node/command_listener.rs).
    - Modules:
        - Command parser:
          - Redis has its own raw tcp protocol command syntax - Redis serialization protocol (RESP).
          - The command parser was done in [./src/resp_parser/commands.rs](./src/resp_parser/commands.rs) (`parse_resp_proc_command()`).
          - The response parser was done in [./src/resp_parser/responses.rs](./src/resp_parser/responses.rs) (`parse_redis_resp_proc_response()`).
        - Command handlers:
          - As of now, all command handlers were implemented in [./src/node/command_handlers.rs](./src/node/command_handlers.rs).
- Replication:
  - Replica to master handshake is implemented in [./src/node/replica_handshake.rs](./src/node/replica_handshake.rs).

---

Made as part of the course by [codecrafters.io](https://app.codecrafters.io/r/joyous-spider-579889).
