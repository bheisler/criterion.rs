use std::fmt::Show;
use std::io::{BufferedReader,Command,PipeStream,Process};

// A two-way channel to the standard streams of a child process
pub struct Stream {
    _process: Process,
    stdin: Option<PipeStream>,
    stdout: BufferedReader<PipeStream>,
}

impl Stream {
    pub fn spawn(cmd: &Command) -> Stream {
        let mut process = match cmd.spawn() {
            Err(e) => fail!("`{}`: {}", cmd, e),
            Ok(process) => process,
        };

        Stream {
            stdin: Some(process.stdin.take_unwrap()),
            stdout: BufferedReader::new(process.stdout.take_unwrap()),
            _process: process,
        }
    }

    pub fn send<T: Show>(&mut self, line: T) {
        match writeln!(self.stdin.get_mut_ref(), "{}", line) {
            Err(e) => fail!("`read stdin`: {}", e),
            Ok(_) => {},
        }
    }

    pub fn recv(&mut self) -> String {
        match self.stdout.read_line() {
            Err(e) => fail!("`write stdout`: {}", e),
            Ok(line) => line,
        }
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        // Close `stdin` first
        drop(self.stdin.take_unwrap());
    }
}
