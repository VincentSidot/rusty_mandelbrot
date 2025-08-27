use log::{LevelFilter, Metadata, Record};
use std::fs::File;
use std::io::Write;
use std::sync::{Mutex, OnceLock};

pub struct LoggerConfig {
    pub level: LevelFilter,
    pub log_to_file: Option<String>,
    pub log_to_console: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LevelFilter::Info,
            log_to_file: None,
            log_to_console: true,
        }
    }
}

struct SimpleLogger {
    config: LoggerConfig,
    file: Option<Mutex<File>>,
}

impl SimpleLogger {
    fn new(config: LoggerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let file = if let Some(ref path) = config.log_to_file {
            Some(Mutex::new(File::create(path)?))
        } else {
            None
        };

        Ok(Self { config, file })
    }
}

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.config.level
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
            let message = format!(
                "[{}] {}:{}: {} - {}",
                record.level(),
                record.file().unwrap_or("unkown"),
                record.line().unwrap_or(0),
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.args()
            );

            // Log to console if enabled
            if self.config.log_to_console {
                println!("{}", message);
            }

            // Log to file if enabled
            if let Some(ref file) = self.file {
                let mut file = file.lock().unwrap();
                let _ = writeln!(file, "{}", message);
                let _ = file.flush();
            }
        }
    }

    fn flush(&self) {
        if let Some(ref file) = self.file {
            let mut file = file.lock().unwrap();
            let _ = file.flush();
        }
    }
}

static LOGGER: OnceLock<Option<SimpleLogger>> = OnceLock::new();

pub fn init(config: LoggerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut is_error = None;

    let logger = LOGGER.get_or_init(|| match SimpleLogger::new(config) {
        Ok(logger) => Some(logger),
        Err(err) => {
            is_error = Some(err);
            None
        }
    });

    if let Some(error) = is_error.take() {
        return Err(error);
    }

    log::set_logger(logger.as_ref().unwrap())
        .map(|_| log::set_max_level(logger.as_ref().unwrap().config.level))?;

    Ok(())
}

pub fn init_default() -> Result<(), Box<dyn std::error::Error>> {
    init(LoggerConfig::default())
}
