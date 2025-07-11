mod cli;
mod connection;
mod client;
mod sge;
mod overlap;

use cli::Args;
use network_direct::Framework;
use std::{fs, net::SocketAddr};
use crate::client::Client;

fn main() {
    let log_file= "logs/client.log";
    let _ = fs::remove_file(log_file);
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout()) // 控制台
        .chain(fern::log_file(log_file).unwrap()) // 文件
        .apply()
        .unwrap();
    let args = Args::parse_args();
    let ip = args.ip;
    let port = args.port;
    let addr = SocketAddr::new(ip, port);
    let adapter = Framework::new().open_adapter(addr).unwrap();
    let mut client = Client::new(adapter);
    client.run(addr);
}
