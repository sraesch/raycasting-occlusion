use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use log::{LevelFilter, info};

/// Workaround for parsing the different log level
#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
        }
    }
}

/// CLI interface for benchmarking and testing the raycasting algorithm for occlusion culling.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    /// The log level
    #[arg(short, value_enum, long, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,

    /// The path to the input file
    #[arg(short, long)]
    pub input: PathBuf,

    /// The size of the image
    #[arg(short, long)]
    pub size: usize,
}

impl Options {
    /// Dumps the options to the log.
    pub fn dump_to_log(&self) {
        info!("Log Level: {:?}", self.log_level);
        info!("Input file: {:?}", self.input);
        info!("Image size: {:?}", self.size);
    }
}
