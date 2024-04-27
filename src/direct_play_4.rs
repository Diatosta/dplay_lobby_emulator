use crate::dpname::DPName;
use std::ffi::c_void;
use windows::core::*;
use windows::Win32::Foundation::BOOL;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

const CLSID_DIRECT_PLAY: GUID = GUID::from_u128(0xD1EB6D20_8923_11D0_9D97_00A0C90A43CB);

#[interface("0AB1C531-4745-11D1-A7A1-0000F803ABFC")]
pub unsafe trait IDirectPlay4A: IUnknown {
    unsafe fn add_player_to_group(&self, id_group: u32, id_player: u32) -> HRESULT;
    unsafe fn close(&self) -> HRESULT;
    unsafe fn create_group(
        &self,
        lpid_group: *mut u32,
        lp_group_name: *const DPName,
        lp_data: *const u8,
        data_size: u32,
        flags: u32,
    ) -> HRESULT;
    unsafe fn create_player(&self) -> HRESULT;
    unsafe fn delete_player_from_group(&self) -> HRESULT;
    unsafe fn destroy_group(&self) -> HRESULT;
    unsafe fn destroy_player(&self) -> HRESULT;
    unsafe fn enum_group_players(&self) -> HRESULT;
    unsafe fn enum_groups(&self) -> HRESULT;
    unsafe fn enum_players(&self) -> HRESULT;
    unsafe fn enum_sessions(&self) -> HRESULT;
    unsafe fn get_caps(&self) -> HRESULT;
    unsafe fn get_group_data(&self) -> HRESULT;
    unsafe fn get_group_name(&self) -> HRESULT;
    unsafe fn get_message_count(&self) -> HRESULT;
    unsafe fn get_player_address(&self) -> HRESULT;
    unsafe fn get_player_caps(&self) -> HRESULT;
    unsafe fn get_player_data(&self) -> HRESULT;
    unsafe fn get_player_name(&self) -> HRESULT;
    unsafe fn get_session_desc(&self) -> HRESULT;
    unsafe fn initialize(&self) -> HRESULT;
    unsafe fn open(&self) -> HRESULT;
    unsafe fn receive(&self) -> HRESULT;
    unsafe fn send(&self) -> HRESULT;
    unsafe fn set_group_data(&self) -> HRESULT;
    unsafe fn set_group_name(&self) -> HRESULT;
    unsafe fn set_player_data(&self) -> HRESULT;
    unsafe fn set_player_name(&self) -> HRESULT;
    unsafe fn set_session_desc(&self) -> HRESULT;
    unsafe fn add_group_to_group(&self) -> HRESULT;
    unsafe fn create_group_in_group(&self) -> HRESULT;
    unsafe fn delete_group_from_group(&self) -> HRESULT;
    unsafe fn enum_connections(
        &self,
        lp_guid_application: *const GUID,
        lp_enum_callback: extern "system" fn(
            *const GUID,
            *const c_void,
            u32,
            *const DPName,
            u32,
            *mut c_void,
        ) -> BOOL,
        lp_context: *mut c_void,
        dw_flags: u32,
    ) -> HRESULT;
}

pub struct DirectPlay4A {
    pub dp: IDirectPlay4A,
}

impl DirectPlay4A {
    pub fn new() -> Result<Self> {
        let dp = unsafe { CoCreateInstance(&CLSID_DIRECT_PLAY, None, CLSCTX_ALL) }?;
        Ok(Self { dp })
    }

    pub fn enum_connections<T>(
        &self,
        guid_application: *const GUID,
        enum_callback: extern "system" fn(
            *const GUID,
            *const c_void,
            u32,
            *const DPName,
            u32,
            *mut c_void,
        ) -> BOOL,
        context: *mut T,
        flags: u32,
    ) -> Result<()> {
        unsafe {
            let context = std::mem::transmute::<*mut T, *mut c_void>(context);

            self.dp
                .enum_connections(guid_application, enum_callback, context, flags)
        }
        .ok()
    }
}
