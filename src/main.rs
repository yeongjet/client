mod cli;
mod client;
mod connection;
mod pixel;
mod sge;
mod r#type;
mod window;

use cli::Args;
use network_direct::{Framework, get_local_addr};
use std::{fs, net::SocketAddr};

use crate::{client::Client, r#type::buffer_size, window::Window};

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
    let adapter = Framework::new().open_adapter(local_addr).unwrap();
    let window_list = vec![Window::<buffer_size::Window1>::new(
        "test Window".to_string(),
    )];
    let mut client = Client::new(adapter, window_list);
    client.run(local_addr, remote_addr);
}
