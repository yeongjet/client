use crate::{
    cli,
    connection::{Connection, ConnectionTrait},
    pixel::Pixel,
    r#type::Overlap,
    window::{self, Window},
};
use log::info;
use network_direct::{Adapter, CompletionQueue, MemoryRegion, NotifyType, RegisterFlags};
use std::{
    fs::File,
    mem,
    net::SocketAddr,
    pin::Pin,
    sync::{Arc, RwLock},
};

pub struct Client
{
    adapter: Adapter,
    adapter_file: File,
    send_cq: CompletionQueue,
    recv_cq: CompletionQueue,
    conn_list: Vec<Pin<Box<dyn ConnectionTrait>>>,
}

impl Client
// where
//     N: ArrayLength,
{
    pub fn new(adapter: Adapter) -> Self {
        let adapter_file = adapter.create_adapter_file().unwrap();
        //  let buffer2 = arr![Pixel::default(); buffer_size::Total];
        //        // let buffer2 = vec![Pixel::default(); 123];

        // let buffer = Box::pin(buffer2);
        // println!("{:p}", buffer);
        // let buffer = Box::pin(arr![Pixel::default(); N]);
        // let mem_region = adapter.create_memory_region(&adapter_file, buffer).unwrap();
        // mem_region
        //     .register(RegisterFlags::ALLOW_LOCAL_WRITE, &mut Overlap::default())
        //     .unwrap();
        let adapter_info = adapter.query().unwrap();
        let queue_depth = std::cmp::min(
            adapter_info.MaxCompletionQueueDepth,
            adapter_info.MaxReceiveQueueDepth,
        );
        let send_cq = adapter
            .create_completion_queue(&adapter_file, queue_depth, 0, 0)
            .unwrap();
        let recv_cq = adapter
            .create_completion_queue(&adapter_file, queue_depth, 0, 0)
            .unwrap();
        let mut send_cq_notify_ov = Box::pin(Overlap::default());
        send_cq
            .notify(NotifyType::Any, &mut *send_cq_notify_ov)
            .unwrap();
        let mut recv_cq_notify_ov = Box::pin(Overlap::default());
        recv_cq
            .notify(NotifyType::Any, &mut *recv_cq_notify_ov)
            .unwrap();
        // let conn_recv_ov = Box::pin(Overlap::default());
        // let conn_recv_ov_ptr = &*conn_recv_ov as *const Overlap;
        // let send_cq_notify_ov_ptr = &*send_cq_notify_ov as *const Overlap;
        // let recv_cq_notify_ov_ptr = &*recv_cq_notify_ov as *const Overlap;
        Self {
            adapter,
            adapter_file,
            send_cq,
            recv_cq,
           //  mem_region,
            // conn_recv_ov,
            // conn_recv_ov_ptr,
            // send_cq_notify_ov,
            // send_cq_notify_ov_ptr,
            // recv_cq_notify_ov,
            // recv_cq_notify_ov_ptr,
            conn_list: vec![],
        }
    }

    pub fn run<const N: usize>(
        &mut self,
        window_list: Vec<Window<N>>,
        local_addr: SocketAddr,
        remote_addr: SocketAddr,
    ) {
        for (index, window) in window_list.iter().enumerate() {
            // 这里需要根据不同的 N 构造不同类型的 Connection
            // 例如：let conn = Connection::<4>::new(...);
            // 这里只做演示，实际 N 的值需要你根据业务逻辑传入
            let conn = Connection::<N>::new(
                index as u8,
                window.title.clone(),
                &self.adapter,
                &self.adapter_file,
                &self.send_cq,
                &self.recv_cq,
                local_addr,
            );
            self.conn_list.push(Box::pin(conn));
        }
        info!("[client] connecting:{}", remote_addr);
        // let iocp = unsafe {
        //     CreateIoCompletionPort(HANDLE(self.adapter_file.as_raw_handle()), None, 0, 0).unwrap()
        // };
        let conn = self.conn_list.first().unwrap();
        conn.connect(remote_addr);
        // let overlap = ptr as *const Overlap;
        // let is_ok = result.is_ok();
        // if !is_ok {
        //     info!("[server] failed: {:?}", result);
        // }
        // let status = if is_ok { "success" } else { "failure" };
        // if overlap == self.send_cq_notify_ov_ptr {
        //     info!("[server] send_cq_notify:{}", status);
        //     if is_ok {
        //         self.handle_send_cq_notify_success();
        //     }
        // } else if overlap == self.recv_cq_notify_ov_ptr {
        //     info!("[server] recv_cq_notify:{}", status);
        //     if is_ok {
        //         self.handle_receive_cq_notify_success();
        //     }
        // } else if overlap == self.conn_recv_ov_ptr {
        //     info!("[server] connect_receive:{}", status);
        //     if is_ok {
        //         self.handle_connect_receive_success(connector)
        //     }
        // }
    }

    // fn handle_connect_receive_success(&mut self, connector: Connector) {
    //     self.conn_list.push(Box::pin(Connection::new(
    //         self.mem_region.clone(),
    //         &self.adapter,
    //         &self.send_cq,
    //         &self.recv_cq,
    //         connector,
    //     )));
    //     let connection = self.conn_list.last_mut().unwrap();
    //     connection.init();
    // }

    // fn handle_send_cq_notify_success(&mut self) {
    //     loop {
    //         let mut results = [unsafe { mem::zeroed() }];
    //         let count = self.send_cq.get_results(&mut results);
    //         let result = &results[0];
    //         if count == 0 {
    //             self.send_cq
    //                 .notify(NotifyType::Any, &mut *self.send_cq_notify_ov)
    //                 .unwrap();
    //             return;
    //         }
    //         info!(
    //             "[server send cq] {} {:?} {:?} {:?} {:?} {:?}",
    //             count,
    //             result.Status,
    //             result.RequestContext,
    //             result.BytesTransferred,
    //             result.QueuePairContext,
    //             result.RequestType
    //         );
    //         if result.Status != ND_SUCCESS {
    //             error!(
    //                 "send failed with status: {} context: {:?}",
    //                 result.Status, result.RequestContext
    //             );
    //             return;
    //         }
    //     }
    // }

    // fn handle_receive_cq_notify_success(&mut self) {
    //     loop {
    //         let mut results = [unsafe { mem::zeroed() }];
    //         let count = self.recv_cq.get_results(&mut results);
    //         let result = &results[0];
    //         if count == 0 {
    //             self.recv_cq
    //                 .notify(NotifyType::Any, &mut *self.recv_cq_notify_ov)
    //                 .unwrap();
    //             return;
    //         }
    //         if result.Status != ND_SUCCESS && result.Status != ND_CANCELED {
    //             panic!(
    //                 "receive failed with status: {} context: {:?}",
    //                 result.Status, result.RequestContext
    //             );
    //         }
    //         info!(
    //             "[server recv cq] {} {:?} {:?} {:?} {:?} {:?}",
    //             count,
    //             result.Status,
    //             result.RequestContext,
    //             result.BytesTransferred,
    //             result.QueuePairContext,
    //             result.RequestType
    //         );
    //     }
    // }
}
