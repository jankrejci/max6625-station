use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the configuration file
    #[arg(long, default_value_t = String::from("config.toml"))]
    pub config: String,

    /// Calibrate all temperature sensors to the supplied temperature
    #[arg(long)]
    pub calibrate: Option<f64>,
}
