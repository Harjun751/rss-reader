use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt::{Display, Write};
use std::fs::File;
use std::io::Write as _;
use tracing::Level;
use tracing::{field::Visit, Subscriber};
use tracing_subscriber::Layer;

pub struct FileLogger;

impl<S: Subscriber> Layer<S> for FileLogger {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut my = Visitor {
            string: &mut String::new(),
        };
        event.record(&mut my);

        match event.metadata().level() {
            &Level::ERROR => {
                let info = format!("{}\n{}{}\n\n", "=".repeat(100), my.string, "=".repeat(100));
                match &mut File::options().append(true).open("error_log.txt") {
                    Ok(f) => match f.write(info.as_bytes()) {
                        Ok(_) => (),
                        Err(_) => {
                            println!("UNABLE TO PRINT TO FILE! Printing to Stdout.\n{}", info)
                        }
                    },
                    Err(_) => println!("UNABLE TO OPEN FILE! Printing to Stdout.\n{}", info),
                };
            }
            _ => {
                let info = format!("{}\n{}{}\n\n", "=".repeat(100), my.string, "=".repeat(100));
                println!("{info}")
            }
        }
    }
}

pub struct Visitor<'a> {
    string: &'a mut String,
}

impl<'a> Visit for Visitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        write!(self.string, "{} = {:?}\n", field.name(), value).unwrap();
    }
}

#[derive(Debug)]
pub struct DetailedError {
    trace: Backtrace,
    pub desc: String,
    pub friendly_desc: Option<String>, // kwaargs: vec<String>,
}

impl DetailedError {
    pub fn new(error: Box<dyn Error>) -> Self {
        let trace = Backtrace::force_capture();
        let desc = error.to_string();
        DetailedError {
            trace,
            desc,
            friendly_desc: None,
        }
    }
    pub fn new_descriptive(error: Box<dyn Error>, friendly_description: &str) -> Self {
        let trace = Backtrace::force_capture();
        let desc = error.to_string();
        DetailedError {
            trace,
            desc,
            friendly_desc: Some(friendly_description.to_string()),
        }
    }
}

impl Error for DetailedError {}

impl Display for DetailedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n BACKTRACE:\n{}", self.desc, self.trace)
    }
}
