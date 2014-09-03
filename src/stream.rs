use std::fmt::Show;
use std::io::{BufferedReader,Command,PipeStream,Process};

// A two-way channel to the standard streams of a child process
pub struct Stream {
    _process: Process,
    stderr: PipeStream,
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
            stderr: process.stderr.take().unwrap(),
            stdin: Some(process.stdin.take().unwrap()),
            stdout: BufferedReader::new(process.stdout.take().unwrap()),
            _process: process,
        }
    }

    pub fn send<T: Show>(&mut self, line: T) {
        match writeln!(self.stdin.get_mut_ref(), "{}", line) {
            Err(e) => fail!("`write into child stdin`: {}", e),
            Ok(_) => {},
        }
    }

    pub fn recv(&mut self) -> String {
        match self.stdout.read_line() {
            Err(e) => {
                println!("stderr:\n{}", self.stderr.read_to_string().unwrap());

                fail!("`read from child stdout`: {}", e);
            },
            Ok(line) => line,
        }
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        // Close `stdin` first
        drop(self.stdin.take().unwrap());
    }
}
