use std::{
    pin::Pin,
    ptr::{self, addr_of},
    sync::{Arc, RwLock},
};

use log::info;
use network_direct::{
    Adapter, CompletionQueue, Connector, MemoryRegion, ND2Overlapped, QueuePair, ReadLimits,
    RequestContext, SendFlags,
    Win32::{Foundation::CloseHandle, System::Threading::CreateEventA},
    core::PCSTR,
};

use crate::{overlap::Overlap, sge::Sge};

pub struct Connection {
    pub id: usize,
    mem_region: Arc<RwLock<MemoryRegion>>,
    pub connector: Connector,
    pub queue_pair: Pin<Box<QueuePair>>,
    accept_ov: Pin<Box<Overlap>>,
    accept_ov_ptr: *const Overlap,
    disconnect_ov: Pin<Box<Overlap>>,
    disconnect_ov_ptr: *const Overlap,
    notify_disconnect_ov: Pin<Box<Overlap>>,
    notify_disconnect_ov_ptr: *const Overlap,
    ack_times: u8,
}

impl Connection {
    pub fn new(
        mem_region: Arc<RwLock<MemoryRegion>>,
        adapter: &Adapter,
        send_cq: &CompletionQueue,
        recv_cq: &CompletionQueue,
        connector: Connector,
    ) -> Self {
        let queue_pair = Box::pin(
            adapter
                .create_queue_pair(recv_cq, send_cq, 1, 1, 1, 1, 0)
                .unwrap(),
        );
        let mut accept_ov = Box::pin(Overlap::default());
        let accept_ov_ptr = &*accept_ov as *const Overlap;
        let mut disconnect_ov = Box::pin(Overlap::default());
        let disconnect_ov_ptr = &*disconnect_ov as *const Overlap;
        let mut notify_disconnect_ov = Box::pin(Overlap::default());
        let notify_disconnect_ov_ptr = &*notify_disconnect_ov as *const Overlap;
        accept_ov.hEvent = unsafe { CreateEventA(None, false, false, PCSTR(ptr::null())).unwrap() };
        disconnect_ov.hEvent =
            unsafe { CreateEventA(None, false, false, PCSTR(ptr::null())).unwrap() };
        notify_disconnect_ov.hEvent =
            unsafe { CreateEventA(None, false, false, PCSTR(ptr::null())).unwrap() };
        let ack_times = 0;
        Self {
            id: 0,
            mem_region,
            connector,
            queue_pair,
            accept_ov,
            accept_ov_ptr,
            disconnect_ov,
            disconnect_ov_ptr,
            notify_disconnect_ov,
            notify_disconnect_ov_ptr,
            ack_times,
        }
    }

    pub fn init(&mut self) {
        let sge_list = [Sge::new(&mut self.mem_region.write().unwrap())];
        self.id = self as *const Connection as usize;
        self.queue_pair
            .receive(RequestContext(self.id as u128), &sge_list)
            .unwrap();
        info!(
            "[conn {:#x}] init connector_ptr: {:p}",
            self.id, &self.connector,
        );
        self.connector
            .accept(
                &self.queue_pair,
                ReadLimits::default(),
                None,
                &mut *self.accept_ov,
            )
            .unwrap();
        self.connector
            .notify_disconnect(&mut *self.notify_disconnect_ov)
            .unwrap();
    }
}
