use crate::report::BenchmarkId as InternalBenchmarkId;
use serde::de::DeserializeOwned;
use std::convert::TryFrom;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;

#[derive(Debug)]
pub enum MessageError {
    SerializationError(serde_json::Error),
    IoError(std::io::Error),
}
impl From<serde_json::Error> for MessageError {
    fn from(other: serde_json::Error) -> Self {
        MessageError::SerializationError(other)
    }
}
impl From<std::io::Error> for MessageError {
    fn from(other: std::io::Error) -> Self {
        MessageError::IoError(other)
    }
}
impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageError::SerializationError(error) => write!(
                f,
                "Failed to serialize or deserialize message to Criterion.rs benchmark:\n{}",
                error
            ),
            MessageError::IoError(error) => write!(
                f,
                "Failed to read or write message to Criterion.rs benchmark:\n{}",
                error
            ),
        }
    }
}
impl std::error::Error for MessageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MessageError::SerializationError(err) => Some(err),
            MessageError::IoError(err) => Some(err),
        }
    }
}

#[derive(Debug)]
pub struct Connection {
    socket: TcpStream,
    receive_buffer: Vec<u8>,
    send_buffer: Vec<u8>,
}
impl Connection {
    pub fn new(socket: TcpStream) -> Self {
        Connection {
            socket,
            receive_buffer: vec![],
            send_buffer: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn recv<T: DeserializeOwned>(&mut self) -> Result<Option<T>, MessageError> {
        let mut length_buf = [0u8; 4];
        match self.socket.read_exact(&mut length_buf) {
            Err(err) if err.kind() == ErrorKind::UnexpectedEof => return Ok(None),
            Err(err) => return Err(err.into()),
            Ok(val) => val,
        };
        let length = u32::from_be_bytes(length_buf);
        self.receive_buffer.resize(length as usize, 0u8);
        self.socket.read_exact(&mut self.receive_buffer)?;
        let value: T = serde_json::from_slice(&self.receive_buffer)?;
        Ok(Some(value))
    }

    pub fn send(&mut self, message: &OutgoingMessage) -> Result<(), MessageError> {
        self.send_buffer.truncate(0);
        serde_json::to_writer(&mut self.send_buffer, message)?;
        let size = u32::try_from(self.send_buffer.len()).unwrap();
        let length_buf = size.to_be_bytes();
        self.socket.write_all(&length_buf)?;
        self.socket.write_all(&self.send_buffer)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "event")]
pub enum IncomingMessage {}

#[derive(Debug, Serialize)]
#[serde(tag = "event")]
pub enum OutgoingMessage {
    BeginningBenchmark { id: RawBenchmarkId },
    SkippingBenchmark { id: RawBenchmarkId },
}

#[derive(Debug, Serialize)]
pub struct RawBenchmarkId {
    group_id: String,
    function_id: Option<String>,
    value_str: Option<String>,
}
impl From<&InternalBenchmarkId> for RawBenchmarkId {
    fn from(other: &InternalBenchmarkId) -> RawBenchmarkId {
        RawBenchmarkId {
            group_id: other.group_id.clone(),
            function_id: other.function_id.clone(),
            value_str: other.value_str.clone(),
        }
    }
}
