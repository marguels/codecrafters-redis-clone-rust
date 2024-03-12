use crate::resp_parser::RESPType;

pub enum Command {
    Ping,
    Echo(String),
    Get(String),         // Key
    Set(String, String, Option<u64>), // Key, Value, Expiry
    Info,
    Unknown,
}

impl Command {
    pub fn from_resp(resp: RESPType) -> Self {
        match resp {
            RESPType::Array(mut arr) if !arr.is_empty() => {
                let command = arr.remove(0);
                match command {
                    RESPType::SimpleString(s) | RESPType::BulkString(s) => match s.to_uppercase().as_str() {
                        "PING" => Command::Ping,
                        "ECHO" if arr.len() >= 1 => {
                            // Expect one argument for ECHO.
                            match arr.remove(0) {
                                RESPType::BulkString(s) => Command::Echo(s),
                                _ => Command::Unknown,
                            }
                        },
                        "GET" if arr.len() >= 1 => {
                            match arr.remove(0) {
                                RESPType::BulkString(s) => Command::Get(s),
                                _ => Command::Unknown,
                            }
                        },
                        "SET" if arr.len() >= 2 => {
                            let key = arr.remove(0);
                            let value = arr.remove(0);
                            let mut expiry = None;
                            while arr.len() >= 2 {
                                match (arr.remove(0), arr.remove(0)) {
                                    (RESPType::BulkString(s), RESPType::BulkString(value)) if s.to_uppercase() == "PX" => {
                                        expiry = value.parse().ok();
                                    }
                                    _ => break,
                                }
                            }

                            match (key, value) {
                                (RESPType::BulkString(k), RESPType::BulkString(v)) => Command::Set(k, v, expiry),
                                _ => Command::Unknown,
                            }
                        },
                        "INFO" => Command::Info,
                        _ => Command::Unknown,
                    },
                    _ => Command::Unknown,
                }
            },
            _ => Command::Unknown,
        }
    }
}