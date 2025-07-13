mod cli;
mod client;
mod connection;
mod pixel;
mod sge;
mod r#type;
mod window;

use cli::Args;
use log::info;
use network_direct::{Framework, get_local_addr};
use std::{fs, net::SocketAddr};

use client::Client;

use crate::window::Window;

fn setup_logging() {
    let log_file = "logs/client.log";
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
}

fn main() {
    setup_logging();
    let args = Args::parse_args();
    let remote_addr = SocketAddr::new(args.ip, args.port);
    let local_addr = get_local_addr(remote_addr);
    info!("local address: {}", local_addr);
    // TODO: 提示当服务器没有启动导致local_addr不在可用ip列表时
    let adapter = Framework::new().open_adapter(local_addr);
    if adapter.is_none() {
        panic!("服务器可能未开启");
    }
    let window_list = vec![Window::<{ 1920 * 1090 }>::new("test Window".to_string())];
    //  Client::new(adapter);
    let mut client = Client::new(adapter.unwrap());
    client.run(window_list, local_addr, remote_addr);
}
