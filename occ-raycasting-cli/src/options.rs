use clap::{Parser, Subcommand, ValueEnum};
use log::{info, LevelFilter};

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

    /// The input files
    #[arg(short, long)]
    pub input_files: String,

    /// The occlusion test subcommand
    #[command(subcommand)]
    pub occ: OccTestSubcommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum OccTestSubcommand {
    /// Using a simple rasterizer
    Rasterizer(RasterizerOptions),
}

/// The arguments for the rasterizer occlusion test
#[derive(Parser, Debug, Clone)]
pub struct RasterizerOptions {
    /// The output file
    #[arg(short, long)]
    pub image_size: usize,
}

impl Options {
    /// Dumps the options to the log.
    pub fn dump_to_log(&self) {
        info!("Log Level: {:?}", self.log_level);
        info!("Input files: {:?}", self.input_files);

        match &self.occ {
            OccTestSubcommand::Rasterizer(options) => {
                info!("Occ Test: Rasterizer");
                info!("Image Size: {:?}", options.image_size);
            }
        }
    }
}
