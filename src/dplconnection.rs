use crate::dpname::DPName;
use crate::dpsession_desc2::DPSessionDesc2;
use std::ffi::c_void;

use windows::core::GUID;

pub const DPOPEN_JOIN: u32 = 0x00000001;
pub const DPOPEN_CREATE: u32 = 0x00000002;

#[repr(C)]
#[derive(Debug)]
pub struct DPLConnection {
    pub size: u32,
    pub flags: u32,
    pub session_desc: *const DPSessionDesc2,
    pub player_name: *const DPName,
    pub guid_sp: GUID,
    pub address: *mut c_void,
    pub address_size: u32,
}
