use crate::{connection::Connection, overlap::Overlap};
use log::{error, info};
use network_direct::{
    Adapter, CompletionQueue, Connector, Listener, MemoryRegion, NotifyType, QueuePair, ReadLimits,
    RegisterFlags,
    Win32::{
        Foundation::HANDLE,
        System::{
            IO::{CreateIoCompletionPort, GetQueuedCompletionStatus},
            Threading::INFINITE,
        },
    },
    get_local_addr,
    sys::{ND_CANCELED, ND_SUCCESS},
};
use std::{
    fs::{File, OpenOptions},
    mem,
    net::SocketAddr,
    os::windows::io::AsRawHandle,
    pin::Pin,
    ptr::addr_of,
    sync::{Arc, RwLock},
};

pub struct Client {
    mem_region: Arc<RwLock<MemoryRegion>>,
    adapter: Adapter,
    adapter_file: File,
    send_cq: CompletionQueue,
    recv_cq: CompletionQueue,
    listener: Listener,
    conn_recv_ov: Pin<Box<Overlap>>,
    conn_recv_ov_ptr: *const Overlap,
    send_cq_notify_ov: Pin<Box<Overlap>>,
    send_cq_notify_ov_ptr: *const Overlap,
    recv_cq_notify_ov: Pin<Box<Overlap>>,
    recv_cq_notify_ov_ptr: *const Overlap,
    conn_list: Vec<Pin<Box<Connection>>>,
}

impl Client {
    pub fn new(adapter: Adapter) -> Self {
        let adapter_file = adapter.create_adapter_file().unwrap();
        let mem_region = adapter
            .create_memory_region(&adapter_file, Box::pin([0u8; 4096]))
            .unwrap();
        mem_region
            .register(RegisterFlags::ALLOW_LOCAL_WRITE, &mut Overlap::default())
            .unwrap();
        let mem_region = Arc::new(RwLock::new(mem_region));
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
        let listener = adapter.create_listener(&adapter_file).unwrap();
        let conn_recv_ov = Box::pin(Overlap::default());
        let conn_recv_ov_ptr = &*conn_recv_ov as *const Overlap;
        let send_cq_notify_ov_ptr = &*send_cq_notify_ov as *const Overlap;
        let recv_cq_notify_ov_ptr = &*recv_cq_notify_ov as *const Overlap;
        let conn_list = vec![];
        Self {
            mem_region,
            adapter,
            adapter_file,
            send_cq,
            recv_cq,
            listener,
            conn_recv_ov,
            conn_recv_ov_ptr,
            send_cq_notify_ov,
            send_cq_notify_ov_ptr,
            recv_cq_notify_ov,
            recv_cq_notify_ov_ptr,
            conn_list,
        }
    }

    pub fn run(&mut self, remote_addr: SocketAddr) {
        info!("[client] connecting:{}", remote_addr);
        let iocp = unsafe {
            CreateIoCompletionPort(HANDLE(self.adapter_file.as_raw_handle()), None, 0, 0).unwrap()
        };
        let local_addr = get_local_addr(remote_addr);
        let connector = self.adapter.create_connector(&self.adapter_file).unwrap();
        let queue_pair = Box::pin(
            self.adapter
                .create_queue_pair(&self.recv_cq, &self.send_cq, 1, 1, 1, 1, 0)
                .unwrap(),
        );
        connector.bind(local_addr);
        let mut connect_ov = Overlap::default();
        connector.connect(
            &queue_pair,
            remote_addr,
            ReadLimits::default(),
            None,
            &mut Overlap::default(),
        );
        let overlap = ptr as *const Overlap;
        let is_ok = result.is_ok();
        if !is_ok {
            info!("[server] failed: {:?}", result);
        }
        let status = if is_ok { "success" } else { "failure" };
        if overlap == self.send_cq_notify_ov_ptr {
            info!("[server] send_cq_notify:{}", status);
            if is_ok {
                self.handle_send_cq_notify_success();
            }
        } else if overlap == self.recv_cq_notify_ov_ptr {
            info!("[server] recv_cq_notify:{}", status);
            if is_ok {
                self.handle_receive_cq_notify_success();
            }
        } else if overlap == self.conn_recv_ov_ptr {
            info!("[server] connect_receive:{}", status);
            if is_ok {
                self.handle_connect_receive_success(connector)
            }
        }
    }

    fn handle_connect_receive_success(&mut self, connector: Connector) {
        self.conn_list.push(Box::pin(Connection::new(
            self.mem_region.clone(),
            &self.adapter,
            &self.send_cq,
            &self.recv_cq,
            connector,
        )));
        let connection = self.conn_list.last_mut().unwrap();
        connection.init();
    }

    fn handle_send_cq_notify_success(&mut self) {
        loop {
            let mut results = [unsafe { mem::zeroed() }];
            let count = self.send_cq.get_results(&mut results);
            let result = &results[0];
            if count == 0 {
                self.send_cq
                    .notify(NotifyType::Any, &mut *self.send_cq_notify_ov)
                    .unwrap();
                return;
            }
            info!(
                "[server send cq] {} {:?} {:?} {:?} {:?} {:?}",
                count,
                result.Status,
                result.RequestContext,
                result.BytesTransferred,
                result.QueuePairContext,
                result.RequestType
            );
            if result.Status != ND_SUCCESS {
                error!(
                    "send failed with status: {} context: {:?}",
                    result.Status, result.RequestContext
                );
                return;
            }
        }
    }

    fn handle_receive_cq_notify_success(&mut self) {
        loop {
            let mut results = [unsafe { mem::zeroed() }];
            let count = self.recv_cq.get_results(&mut results);
            let result = &results[0];
            if count == 0 {
                self.recv_cq
                    .notify(NotifyType::Any, &mut *self.recv_cq_notify_ov)
                    .unwrap();
                return;
            }
            if result.Status != ND_SUCCESS && result.Status != ND_CANCELED {
                panic!(
                    "receive failed with status: {} context: {:?}",
                    result.Status, result.RequestContext
                );
            }
            info!(
                "[server recv cq] {} {:?} {:?} {:?} {:?} {:?}",
                count,
                result.Status,
                result.RequestContext,
                result.BytesTransferred,
                result.QueuePairContext,
                result.RequestType
            );
        }
    }
}
