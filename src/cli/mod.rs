use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Run is server mode.
    #[arg[short, long]]
    pub serve: bool,

    /// Server port
    #[arg[long]]
    pub port: Option<String>,

    /// Create a new tweet
    #[arg[short, long]]
    pub tweet: Option<String>,
}
