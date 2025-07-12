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
    //setup_logging();
    let args = Args::parse_args();
    let remote_addr = SocketAddr::new(args.ip, args.port);
    let local_addr = get_local_addr(remote_addr);
    info!("local address: {}", local_addr);
    let adapter = Framework::new().open_adapter(local_addr).unwrap();
    // let window_list = vec![Window::<buffer_size::Window1>::new(
    //     "test Window".to_string(),
    // )];
            let adapter_file = adapter.create_adapter_file().unwrap();
    println!("Adapter file created: {:?}", adapter_file);
    Client::new(adapter);
    // let mut client = Client::new(adapter);
   // client.run(window_list, local_addr, remote_addr);
}
