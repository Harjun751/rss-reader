use chrono::Local;
use std::{
    backtrace::Backtrace,
    error::Error,
    fmt::{Debug, Display},
    fs::File,
    io::{prelude::*, SeekFrom},
};
use tracing::{field::Visit, Level, Subscriber};
use tracing_subscriber::Layer;

pub struct FileLogger;

impl<S: Subscriber> Layer<S> for FileLogger {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut my = Visitor {
            val_list: &mut vec![],
        };
        event.record(&mut my);

        match event.metadata().level() {
            &Level::ERROR => {
                match &mut File::options().read(true).write(true).open("error_log.xml") {
                    Ok(file) => {
                        // THERE HAS TO BE A BETTER WAY OF DOING THIS!!!

                        file.seek(SeekFrom::Start(7)).unwrap();
                        // Read data after the position
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer).unwrap();

                        // Move back to the desired position
                        file.seek(SeekFrom::Start(7)).unwrap();

                        // Write the new data
                        file.write_all(my.to_string().as_bytes()).unwrap();

                        // Write back the data that was read
                        file.write_all(&buffer).unwrap();
                    }
                    Err(e) => println!("UNABLE TO OPEN FILE! Printing to Stdout.\n{}", e),
                };
            }
            _ => {
                println!("{}", my.to_string())
            }
        }
    }
}

pub struct Visitor<'a> {
    val_list: &'a mut Vec<KVP<'a>>,
}
struct KVP<'a> {
    pub key: &'a str,
    pub value: String,
}

impl<'a> Visit for Visitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        println!("{}", format!("{:?}", value));
        self.val_list.push(KVP {
            key: field.name(),
            value: format!("{:?}", value),
        });
    }
}

impl<'a> ToString for Visitor<'a> {
    fn to_string(&self) -> String {
        let time = Local::now();
        let mut formatted_string = format!("<error>\n\t<time>{}</time>\n", time);
        for val in self.val_list.iter() {
            let name = val.key;
            formatted_string
                .push_str(format!("\t<{name}>\n\t\t{}\n\t</{name}>\n", val.value).as_str())
        }
        formatted_string.push_str("</error>\n");
        formatted_string
    }
}
pub struct DetailedError {
    trace: Backtrace,
    pub desc: String,
    pub friendly_desc: Option<String>,
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
    pub fn new_with_message(error: &str) -> Self {
        let trace = Backtrace::force_capture();
        DetailedError {
            trace,
            desc: error.to_string(),
            friendly_desc: None,
        }
    }
}

impl Error for DetailedError {}

impl Debug for DetailedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<![CDATA[{}]]>", self.trace.to_string())
    }
}

impl Display for DetailedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<![CDATA[{}]]>", self.trace.to_string())
    }
}

impl From<mysql::Error> for DetailedError {
    fn from(value: mysql::Error) -> Self {
        DetailedError::new(Box::new(value))
    }
}
