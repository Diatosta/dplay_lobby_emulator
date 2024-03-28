use crate::DPLAppInfo;
use std::ffi::c_void;
use windows::core::{interface, IUnknown, IUnknown_Vtbl, GUID, HRESULT};
use windows::Win32::Foundation::{BOOL, HWND};

#[interface("2DB72491-652C-11d1-A7A8-0000F803ABFC")]
pub unsafe trait IDirectPlayLobby3A: IUnknown {
    unsafe fn connect(
        &self,
        dw_flags: u32,
        lp_direct_play_2: *mut c_void,
        p_unk: *const c_void,
    ) -> HRESULT;
    unsafe fn create_address(
        &self,
        guid_sp: *const GUID,
        guid_data_type: *const GUID,
        lp_data: *const c_void,
        dw_data_size: u32,
        lp_address: *mut c_void,
        lpdw_address_size: *mut u32,
    ) -> HRESULT;
    unsafe fn enum_address(
        &self,
        lp_enum_address_callback: *const c_void,
        lp_address: *const c_void,
        dw_address_size: u32,
        lp_context: *mut c_void,
    ) -> HRESULT;
    unsafe fn enum_address_types(
        &self,
        lp_enum_address_type_callback: *const c_void,
        guid_sp: *const GUID,
        lp_context: *mut c_void,
        dw_flags: u32,
    ) -> HRESULT;
    unsafe fn enum_local_applications(
        &self,
        lp_enum_local_app_callback: extern "system" fn(
            *const DPLAppInfo,
            *const c_void,
            u32,
        ) -> BOOL,
        lp_context: HWND,
        dw_flags: u32,
    ) -> HRESULT;
}

pub unsafe fn enum_local_applications(
    direct_play_3: &IDirectPlayLobby3A,
    lp_enum_local_app_callback: extern "system" fn(*const DPLAppInfo, *const c_void, u32) -> BOOL,
    lp_context: HWND,
    dw_flags: u32,
) -> HRESULT {
    direct_play_3.enum_local_applications(lp_enum_local_app_callback, lp_context, dw_flags)
}
