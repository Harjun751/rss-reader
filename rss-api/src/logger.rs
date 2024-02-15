use std::fmt::Write;
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
        println!(
            "Got event\n level={:?}\n target={:?}\n name={:?}",
            event.metadata().level(),
            event.metadata().target(),
            event.metadata().name()
        );
        println!("Visitor recorded fields: {}", my.string);
    }
}

pub struct Visitor<'a> {
    string: &'a mut String,
}

impl<'a> Visit for Visitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        write!(self.string, "{} = {:?};", field.name(), value).unwrap();
    }
}
