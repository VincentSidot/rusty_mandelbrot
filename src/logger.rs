use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        // Discard logs from wgpu
        if record.target().starts_with("wgpu") {
            return;
        }
        // Discard logs from naga
        if record.target().starts_with("naga") {
            return;
        }
        // Discard logs from egui
        if record.target().starts_with("egui") {
            return;
        }

        // Discard logs from winit
        if record.target().starts_with("winit") {
            return;
        }

        if self.enabled(record.metadata()) {
            println!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(level))
}
