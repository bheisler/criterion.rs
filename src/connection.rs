use crate::report::BenchmarkId as InternalBenchmarkId;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::io::{Read, Write};
use std::mem::size_of;
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

const MAGIC_NUMBER: &str = "Criterion";
const HELLO_SIZE: usize = MAGIC_NUMBER.len() // magic number
    + (size_of::<u8>() * 3) // criterion.rs version
    + size_of::<u16>() // protocol version
    + size_of::<u16>(); // protocol format
const PROTOCOL_VERSION: u16 = 1;
const PROTOCOL_FORMAT: u16 = 1;

#[derive(Debug)]
struct InnerConnection {
    socket: TcpStream,
    receive_buffer: Vec<u8>,
    send_buffer: Vec<u8>,
}
impl InnerConnection {
    pub fn new(mut socket: TcpStream) -> Result<Self, std::io::Error> {
        // Send the connection hello message right away.
        let mut hello_buf = [0u8; HELLO_SIZE];
        let mut i = 0usize;
        hello_buf[i..i + MAGIC_NUMBER.len()].copy_from_slice(MAGIC_NUMBER.as_bytes());
        i += MAGIC_NUMBER.len();
        hello_buf[i] = env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap();
        hello_buf[i + 1] = env!("CARGO_PKG_VERSION_MINOR").parse().unwrap();
        hello_buf[i + 2] = env!("CARGO_PKG_VERSION_PATCH").parse().unwrap();
        i += 3;
        hello_buf[i..i + 2].clone_from_slice(&PROTOCOL_VERSION.to_be_bytes());
        i += 2;
        hello_buf[i..i + 2].clone_from_slice(&PROTOCOL_FORMAT.to_be_bytes());

        socket.write_all(&hello_buf)?;

        Ok(InnerConnection {
            socket,
            receive_buffer: vec![],
            send_buffer: vec![],
        })
    }

    #[allow(dead_code)]
    pub fn recv(&mut self) -> Result<IncomingMessage, MessageError> {
        let mut length_buf = [0u8; 4];
        self.socket.read_exact(&mut length_buf)?;
        let length = u32::from_be_bytes(length_buf);
        self.receive_buffer.resize(length as usize, 0u8);
        self.socket.read_exact(&mut self.receive_buffer)?;
        let value = serde_json::from_slice(&self.receive_buffer)?;
        Ok(value)
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

/// This is really just a holder to allow us to send messages through a shared reference to the
/// connection.
#[derive(Debug)]
pub struct Connection {
    inner: RefCell<InnerConnection>,
}
impl Connection {
    pub fn new(socket: TcpStream) -> Result<Self, std::io::Error> {
        Ok(Connection {
            inner: RefCell::new(InnerConnection::new(socket)?),
        })
    }

    #[allow(dead_code)]
    pub fn recv(&self) -> Result<IncomingMessage, MessageError> {
        self.inner.borrow_mut().recv()
    }

    pub fn send(&self, message: &OutgoingMessage) -> Result<(), MessageError> {
        self.inner.borrow_mut().send(message)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "event")]
pub enum IncomingMessage {
    RunBenchmark,
    SkipBenchmark,
    __Other,
}

#[derive(Debug, Serialize)]
#[serde(tag = "event")]
pub enum OutgoingMessage<'a> {
    BeginningBenchmarkGroup {
        group: &'a str,
    },
    FinishedBenchmarkGroup {
        group: &'a str,
    },
    BeginningBenchmark {
        id: RawBenchmarkId,
    },
    SkippingBenchmark {
        id: RawBenchmarkId,
    },
    Warmup {
        id: RawBenchmarkId,
        nanos: f64,
    },
    MeasurementStart {
        id: RawBenchmarkId,
        sample_count: u64,
        estimate_ns: f64,
        iter_count: u64,
    },
    MeasurementComplete {
        id: RawBenchmarkId,
        iters: &'a [u64],
        times: &'a [f64],
    },
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
