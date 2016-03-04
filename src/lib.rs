use std::time::Duration;
use std::fmt;

pub mod function;
pub mod input;
pub mod program;

pub struct Bencher {}

struct Config {
    measurement_time: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            measurement_time: Duration::from_secs(5),
        }
    }
}

pub struct Criterion<I, S> {
    config: Config,
    input: I,
    /// "Subject" under test, it could be a Function or a Program
    test: S,
}

impl<I, S> Criterion<I, S> {
    pub fn input<T>(self, input: T) -> Criterion<input::One<T>, ()>
        where T: fmt::Display,
    {
        Criterion {
            config: self.config,
            input: input::One { value: input },
            test: (),
        }
    }

    pub fn inputs<T>(self, input: Vec<T>) -> Criterion<input::Many<T>, ()>
        where T: fmt::Display,
    {
        Criterion {
            config: self.config,
            input: input::Many { values: input },
            test: (),
        }
    }

    pub fn measurement_time(self, dur: Duration) -> Criterion<I, S> {
        Criterion {
            config: Config { measurement_time: dur, .. self.config },
            input: self.input,
            test: self.test,
        }
    }
}

impl<S> Criterion<(), S> {
    pub fn function(self, f: Function<()>) -> Criterion<(), function::One<()>> {
        Criterion {
            config: self.config,
            input: (),
            test: function::One { function: f },
        }
    }

    pub fn functions(self, fs: Vec<Function<()>>) -> Criterion<(), function::Many<()>> {
        Criterion {
            config: self.config,
            input: (),
            test: function::Many { functions: fs },
        }
    }
}

impl Criterion<(), function::Many<()>> {
    pub fn bench(self) -> Criterion<(), ()> {
        let ref mut b = Bencher {};

        for mut f in self.test.functions {
            (f.call)(b, &());
            println!("Benchmarking {}", f.name);
        }

        Criterion {
            config: self.config,
            input: (),
            test: (),
        }
    }
}

impl Criterion<(), function::One<()>> {
    pub fn bench(mut self) -> Criterion<(), ()> {
        let ref mut b = Bencher {};

        (self.test.function.call)(b, &());
        println!("Benchmarking {}", self.test.function.name);

        Criterion {
            config: self.config,
            input: (),
            test: (),
        }
    }
}

impl<T, S> Criterion<input::One<T>, S> {
    pub fn function(self, f: Function<T>) -> Criterion<input::One<T>, function::One<T>> {
        Criterion {
            config: self.config,
            input: self.input,
            test: function::One { function: f },
        }
    }

    pub fn functions(self, fs: Vec<Function<T>>) -> Criterion<input::One<T>, function::Many<T>> {
        Criterion {
            config: self.config,
            input: self.input,
            test: function::Many { functions: fs },
        }
    }
}

impl<T, S> Criterion<input::Many<T>, S> {
    pub fn function(self, f: Function<T>) -> Criterion<input::Many<T>, function::One<T>> {
        Criterion {
            config: self.config,
            input: self.input,
            test: function::One { function: f },
        }
    }

    pub fn functions(self, fs: Vec<Function<T>>) -> Criterion<input::Many<T>, function::Many<T>> {
        Criterion {
            config: self.config,
            input: self.input,
            test: function::Many { functions: fs },
        }
    }
}

impl<T> Criterion<input::Many<T>, function::Many<T>>
    where T: fmt::Display,
{
    pub fn bench(self) -> Criterion<(), ()> {
        let ref mut b = Bencher {};

        for mut f in self.test.functions {
            for input in &self.input.values {
                (f.call)(b, input);
                println!("Benchmarking {} with input {}", f.name, input);
            }
        }

        Criterion {
            config: self.config,
            input: (),
            test: (),
        }
    }
}

impl<T> Criterion<input::Many<T>, function::One<T>>
    where T: fmt::Display,
{
    pub fn bench(mut self) -> Criterion<(), ()> {
        let ref mut b = Bencher {};

        for ref input in self.input.values {
            (self.test.function.call)(b, input);
            println!("Benchmarking {} with input {}", self.test.function.name, input);
        }

        Criterion {
            config: self.config,
            input: (),
            test: (),
        }
    }
}

impl<T> Criterion<input::One<T>, function::One<T>>
    where T: fmt::Display,
{
    pub fn bench(mut self) -> Criterion<(), ()> {
        let ref mut b = Bencher {};

        (self.test.function.call)(b, &self.input.value);
        println!("Benchmarking {} with input {}", self.test.function.name, self.input.value);

        Criterion {
            config: self.config,
            input: (),
            test: (),
        }
    }
}

impl<T> Criterion<input::One<T>, function::Many<T>>
    where T: fmt::Display,
{
    pub fn bench(self) -> Criterion<(), ()> {
        let ref mut b = Bencher {};

        for mut f in self.test.functions {
            (f.call)(b, &self.input.value);
            println!("Benchmarking {} with input {}", f.name, self.input.value);
        }

        Criterion {
            config: self.config,
            input: (),
            test: (),
        }
    }
}

impl Default for Criterion<(), ()> {
    fn default() -> Self {
        Criterion {
            config: Config::default(),
            input: (),
            test: (),
        }
    }
}

/// Named function
pub struct Function<T> {
    name: String,
    call: Box<FnMut(&mut Bencher, &T)>,
}

#[allow(non_snake_case)]
pub fn Function<T, S, F>(name: S, f: F) -> Function<T>
    where F: FnMut(&mut Bencher, &T) + 'static,
          S: Into<String>,
{
        Function {
            name: name.into(),
            call: Box::new(f),
        }
}
