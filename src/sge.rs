use network_direct::{
    MemoryRegion,
    sys::{ND2_SGE, UINT32, ULONG},
};

use crate::pixel::Pixel;

pub struct Sge {
    pub Buffer: *mut ::std::os::raw::c_void,
    pub BufferLength: ULONG,
    pub MemoryRegionToken: UINT32,
}

impl Sge {
    pub fn new(buffer: &mut [Pixel], token: u32) -> ND2_SGE {
        ND2_SGE {
            Buffer: buffer.as_mut_ptr() as *mut std::ffi::c_void,
            BufferLength: buffer.len() as u32,
            MemoryRegionToken: token,
        }
    }
}
