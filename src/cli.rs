use std::net::IpAddr;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "server")]
#[command(about = "Screen capturer server application")]
#[command(version = "1.0")]
pub struct Args {
    /// IP address to bind the server to
    #[arg(short = 'a', long = "address", default_value = "127.0.0.1")]
    pub ip: IpAddr,

    /// Port number to run the server on
    #[arg(short = 'p', long = "port", default_value = "3421")]
    pub port: u16,
}

impl Args {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Args::parse()
    }
}
