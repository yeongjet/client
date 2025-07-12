use std::{
    fs::File,
    net::SocketAddr,
    pin::Pin,
    sync::{Arc, RwLock},
};

use generic_array::{ArrayLength, GenericArray};
use network_direct::{
    Adapter, BindFlags, Buffer, CompletionQueue, Connector, MemoryRegion, MemoryWindow, QueuePair,
    ReadLimits, RemoteToken, RequestContext, WriteFlags,
};

use crate::{
    pixel::Pixel,
    sge::Sge,
    r#type::{Overlap, buffer_size},
};

pub struct Connection<'a> {
    pub index: u8,
    title: String,
    mem_region: &'a mut MemoryRegion<Pixel, buffer_size::Total>,
    pub mem_window: Pin<Box<MemoryWindow>>,
    pub connector: Connector,
    pub queue_pair: Pin<Box<QueuePair>>,
    accept_ov: Pin<Box<Overlap>>,
    accept_ov_ptr: *const Overlap,
    disconnect_ov: Pin<Box<Overlap>>,
    disconnect_ov_ptr: *const Overlap,
    notify_disconnect_ov: Pin<Box<Overlap>>,
    notify_disconnect_ov_ptr: *const Overlap,
    buffer_range: (usize, usize),
    remote_token: Option<RemoteToken>,
}

impl<'a> Connection<'a> {
    pub fn new(
        index: u8,
        title: String,
        mem_region: &'a mut MemoryRegion<Pixel, buffer_size::Total>,
        adapter: &Adapter,
        adapter_file: &File,
        send_cq: &CompletionQueue,
        recv_cq: &CompletionQueue,
        local_addr: SocketAddr,
        buffer_range: (usize, usize),
    ) -> Self {
        let connector = adapter.create_connector(adapter_file).unwrap();
        connector.bind(local_addr).unwrap();
        let mem_window = Box::pin(adapter.create_memory_window().unwrap());
        let queue_pair = Box::pin(
            adapter
                .create_queue_pair(recv_cq, send_cq, 1, 1, 1, 1, 0)
                .unwrap(),
        );
        let buffer = mem_region.buffer[buffer_range.0..buffer_range.1].as_ref();
        queue_pair.bind(
            RequestContext(index as u128),
            mem_region,
            &*mem_window.as_ref(),
            buffer,
            BindFlags::ALLOW_WRITE | BindFlags::ALLOW_READ,
        );
        let accept_ov = Box::pin(Overlap::default());
        let accept_ov_ptr = &*accept_ov as *const Overlap;
        let disconnect_ov = Box::pin(Overlap::default());
        let disconnect_ov_ptr = &*disconnect_ov as *const Overlap;
        let notify_disconnect_ov: Pin<Box<network_direct::Win32::System::IO::OVERLAPPED>> =
            Box::pin(Overlap::default());
        let notify_disconnect_ov_ptr = &*notify_disconnect_ov as *const Overlap;
        Self {
            index,
            title,
            mem_region,
            mem_window,
            connector,
            queue_pair,
            accept_ov,
            accept_ov_ptr,
            disconnect_ov,
            disconnect_ov_ptr,
            notify_disconnect_ov,
            notify_disconnect_ov_ptr,
            buffer_range,
            remote_token: None,
        }
    }

    pub fn connect(&self, remote_addr: SocketAddr) {
        // let sge_list = [Sge::new(&mut self.mem_region)];
        // let sge = ND2_SGE {
        //     Buffer: buffer.as_mut_ptr() as *mut std::ffi::c_void,
        //     BufferLength: buffer.len() as u32,
        //     MemoryRegionToken: mem_region.get_remote_token().0,
        // };

        self.connector
            .connect(
                &self.queue_pair,
                remote_addr,
                ReadLimits::default(),
                None,
                &mut Overlap::default(),
            )
            .unwrap();
        self.connector
            .complete_connect(&mut Overlap::default())
            .unwrap();
    }

    pub fn write(&mut self) {
        // let buffer = *self.mem_region.buffer_mut();
        // let p = &mut buffer[..];
        let (start, end) = self.buffer_range;
        let remote_token = self.mem_region.get_remote_token();
        let buffer = self.mem_region.buffer[start..end].as_mut();
        let sgl = [Sge::new(buffer, remote_token)];
        self.queue_pair.write(
            RequestContext(self.index as u128),
            &sgl,
            0,
            self.mem_window.remote_token(),
            WriteFlags::SILENT_SUCCESS,
        );
    }
}
