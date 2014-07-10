use std::io::{BufferedReader,Command,PipeStream,Process};

pub struct Bencher {
    _child: Process,
    iterations: u64,
    ns_elapsed: u64,
    stdin: Option<PipeStream>,
    stdout: BufferedReader<PipeStream>,
}

impl Bencher {
    pub fn new(cmd: Command) -> Bencher {
        let mut child = cmd.spawn().unwrap();
        let stdin = Some(child.stdin.take_unwrap());
        let stdout = BufferedReader::new(child.stdout.take_unwrap());

        Bencher {
            _child: child,
            iterations: 0,
            ns_elapsed: 0,
            stdin: stdin,
            stdout: stdout,
        }
    }

    pub fn bench_n(&mut self, n: u64) {
        self.iterations = n;
        (writeln!(self.stdin.get_mut_ref(), "{}", n)).unwrap();

        let line = self.stdout.read_line().unwrap();
        self.ns_elapsed = from_str(line.as_slice().trim()).unwrap();
    }

    pub fn ns_elapsed(&self) -> u64 {
        self.ns_elapsed
    }

    pub fn ns_per_iter(&self) -> f64 {
        self.ns_elapsed as f64 / self.iterations as f64
    }
}

impl Drop for Bencher {
    fn drop(&mut self) {
        // Close `stdin` first
        drop(self.stdin.take_unwrap())
    }
}
