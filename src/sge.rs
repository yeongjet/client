use network_direct::{
    MemoryRegion,
    sys::{ND2_SGE, UINT32, ULONG},
};

pub struct Sge {
    pub Buffer: *mut ::std::os::raw::c_void,
    pub BufferLength: ULONG,
    pub MemoryRegionToken: UINT32,
}

impl Sge {
    pub fn new(mem_region: &mut MemoryRegion) -> ND2_SGE {
        let mut buffer = mem_region.buffer_mut();
        ND2_SGE {
            Buffer: buffer.as_mut_ptr() as *mut std::ffi::c_void,
            BufferLength: buffer.len() as u32,
            MemoryRegionToken: mem_region.get_local_token().0,
        }
    }
}
