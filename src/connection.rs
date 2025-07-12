use std::{
    fs::File,
    net::SocketAddr,
    pin::Pin,
    sync::{Arc, RwLock},
};

use generic_array::{ArrayLength, GenericArray, arr};
use network_direct::{
    Adapter, BindFlags, Buffer, CompletionQueue, Connector, MemoryRegion, MemoryWindow, QueuePair,
    ReadLimits, RegisterFlags, RemoteToken, RequestContext, WriteFlags,
};

use crate::{
    pixel::Pixel,
    sge::Sge,
    r#type::{Overlap, buffer_size},
};

pub struct Connection<N>
where
    N: ArrayLength,
{
    pub index: u8,
    title: String,
    //buffer: Pin<Box<GenericArray<Pixel, N>>>,
    mem_region: MemoryRegion<Pixel, N>,
    pub mem_window: Pin<Box<MemoryWindow>>,
    pub connector: Connector,
    pub queue_pair: Pin<Box<QueuePair>>,
    accept_ov: Pin<Box<Overlap>>,
    accept_ov_ptr: *const Overlap,
    disconnect_ov: Pin<Box<Overlap>>,
    disconnect_ov_ptr: *const Overlap,
    notify_disconnect_ov: Pin<Box<Overlap>>,
    notify_disconnect_ov_ptr: *const Overlap,

    remote_token: Option<RemoteToken>,
}

impl<N: ArrayLength> Connection<N> {
    pub fn new(
        index: u8,
        title: String,
        adapter: &Adapter,
        adapter_file: &File,
        send_cq: &CompletionQueue,
        recv_cq: &CompletionQueue,
        local_addr: SocketAddr,
    ) -> Self {
        let connector = adapter.create_connector(adapter_file).unwrap();
        connector.bind(local_addr).unwrap();
        let mem_window = Box::pin(adapter.create_memory_window().unwrap());
        let queue_pair = Box::pin(
            adapter
                .create_queue_pair(recv_cq, send_cq, 1, 1, 1, 1, 0)
                .unwrap(),
        );
        let buffer = Box::pin(GenericArray::<Pixel, N>::default());
        // let buffer = Box::pin(arr![Pixel::default(); N]);
        let mem_region = adapter.create_memory_region(&adapter_file, buffer).unwrap();
        mem_region
            .register(RegisterFlags::ALLOW_LOCAL_WRITE, &mut Overlap::default())
            .unwrap();
        queue_pair.bind(
            RequestContext(index as u128),
            &mem_region,
            &*mem_window.as_ref(),
            &mem_region.buffer,
            BindFlags::ALLOW_WRITE | BindFlags::ALLOW_READ,
        ).unwrap();
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
        let remote_token = self.mem_region.get_remote_token();
        let buffer = unsafe {
            self.mem_region
                .buffer
                .as_mut()
                .get_unchecked_mut()
                .as_mut_slice()
        };
        // let buffer = self.mem_region.buffer.as_mut_slice();

        //let mut buffer = self.mem_region.buffer.as_mut();
        let sgl = [Sge::new(&mut buffer.as_mut(), remote_token)];
        self.queue_pair.write(
            RequestContext(self.index as u128),
            &sgl,
            0,
            self.mem_window.remote_token(),
            WriteFlags::SILENT_SUCCESS,
        );
    }
}
