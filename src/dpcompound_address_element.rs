use std::ffi::c_void;
use windows::core::GUID;

#[repr(C)]
#[derive(Debug)]
pub struct DPCompoundAddressElement {
    pub guid_data_type: GUID,
    pub data_size: u32,
    pub data: *const c_void,
}
