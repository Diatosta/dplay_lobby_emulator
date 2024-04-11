use crate::dpcompound_address_element::DPCompoundAddressElement;
use crate::dplconnection::DPLConnection;
use crate::DPLAppInfo;
use std::ffi::c_void;
use windows::core::{interface, IUnknown, IUnknown_Vtbl, GUID, HRESULT, Error};
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
        lp_enum_address_type_callback: extern "system" fn(*const GUID, *const c_void, u32) -> BOOL,
        guid_sp: *mut GUID,
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
    unsafe fn get_connection_settings(&self) -> HRESULT;
    unsafe fn receive_lobby_message(&self) -> HRESULT;
    unsafe fn run_application(
        &self,
        flags: u32,
        dw_app_id: *mut u32,
        lp_connection: *const DPLConnection,
        receive_event: *const c_void,
    ) -> HRESULT;
    unsafe fn send_lobby_message(&self) -> HRESULT;
    unsafe fn set_connection_settings(&self) -> HRESULT;
    unsafe fn set_lobby_message(&self) -> HRESULT;
    unsafe fn create_compound_address(
        &self,
        elements: *const DPCompoundAddressElement,
        element_count: u32,
        address: *mut c_void,
        address_size: *mut u32,
    ) -> HRESULT;
}

pub unsafe fn enum_address_types(
    direct_play_3: &IDirectPlayLobby3A,
    lp_enum_address_type_callback: extern "system" fn(*const GUID, *const c_void, u32) -> BOOL,
    guid_sp: *mut GUID,
    lp_context: *mut c_void,
    dw_flags: u32,
) -> HRESULT {
    direct_play_3.enum_address_types(lp_enum_address_type_callback, guid_sp, lp_context, dw_flags)
}

pub unsafe fn enum_local_applications(
    direct_play_3: &IDirectPlayLobby3A,
    lp_enum_local_app_callback: extern "system" fn(*const DPLAppInfo, *const c_void, u32) -> BOOL,
    lp_context: HWND,
    dw_flags: u32,
) -> HRESULT {
    direct_play_3.enum_local_applications(lp_enum_local_app_callback, lp_context, dw_flags)
}

pub unsafe fn create_address(
    direct_play_3: &IDirectPlayLobby3A,
    guid_sp: *const GUID,
    guid_data_type: *const GUID,
    lp_data: *const c_void,
    dw_data_size: u32,
    lp_address: *mut c_void,
    lpdw_address_size: *mut u32,
) -> HRESULT {
    direct_play_3.create_address(
        guid_sp,
        guid_data_type,
        lp_data,
        dw_data_size,
        lp_address,
        lpdw_address_size,
    )
}

pub fn run_application(
    direct_play_3: &IDirectPlayLobby3A,
    flags: u32,
    dw_app_id: *mut u32,
    lp_connection: *const DPLConnection,
    receive_event: *const c_void,
) -> Result<(), Error> {
    unsafe { direct_play_3.run_application(flags, dw_app_id, lp_connection, receive_event) }.ok()
}

pub unsafe fn create_compound_address(
    direct_play_3: &IDirectPlayLobby3A,
    elements: *const DPCompoundAddressElement,
    element_count: u32,
    address: *mut c_void,
    address_size: *mut u32,
) -> HRESULT {
    direct_play_3.create_compound_address(elements, element_count, address, address_size)
}
