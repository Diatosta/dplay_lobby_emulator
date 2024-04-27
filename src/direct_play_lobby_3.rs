use crate::dpcompound_address_element::DPCompoundAddressElement;
use crate::dplconnection::DPLConnection;
use crate::DPLAppInfo;
use std::ffi::c_void;
use windows::core::*;
use windows::Win32::Foundation::BOOL;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

const CLSID_DIRECT_PLAY_LOBBY: GUID = GUID::from_u128(0x2FE8F810_B2A5_11D0_A787_0000F803ABFC);

pub const DPAID_SERVICE_PROVIDER: GUID = GUID::from_u128(0x07D916C0_E0AF_11CF_9C4E_00A0C905425E);
pub const DPAID_INET: GUID = GUID::from_u128(0xC4A54DA0_E0AF_11CF_9C4E_00A0C905425E);
pub const DPAID_PHONE: GUID = GUID::from_u128(0x78EC89A0_E0AF_11CF_9C4E_00A0C905425E);

#[interface("2DB72491-652C-11d1-A7A8-0000F803ABFC")]
pub unsafe trait IDirectPlayLobby3A: IUnknown {
    unsafe fn connect(
        &self,
        dw_flags: u32,
        lp_direct_play_2: *mut c_void,
        p_unk: *const c_void,
    ) -> HRESULT;
    unsafe fn create_address(&self) -> HRESULT;
    unsafe fn enum_address(
        &self,
        lp_enum_address_callback: *const c_void,
        lp_address: *const c_void,
        dw_address_size: u32,
        lp_context: *mut c_void,
    ) -> HRESULT;
    unsafe fn enum_address_types(
        &self,
        lp_enum_address_type_callback: extern "system" fn(*const GUID, *mut c_void, u32) -> BOOL,
        guid_sp: *mut GUID,
        lp_context: *mut c_void,
        dw_flags: u32,
    ) -> HRESULT;
    unsafe fn enum_local_applications(
        &self,
        lp_enum_local_app_callback: extern "system" fn(*const DPLAppInfo, *mut c_void, u32) -> BOOL,
        lp_context: *mut c_void,
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

pub struct DirectPlayLobby3A {
    pub dp_lobby: IDirectPlayLobby3A,
}

impl DirectPlayLobby3A {
    pub fn new() -> Result<Self> {
        let dp_lobby = unsafe { CoCreateInstance(&CLSID_DIRECT_PLAY_LOBBY, None, CLSCTX_ALL) }?;
        Ok(Self { dp_lobby })
    }

    pub fn enum_address_types<T>(
        &self,
        lp_enum_address_type_callback: extern "system" fn(*const GUID, *mut c_void, u32) -> BOOL,
        guid_sp: &mut GUID,
        context: &mut T,
        flags: u32,
    ) -> Result<()> {
        let context = unsafe { std::mem::transmute::<*mut T, *mut c_void>(context) };

        self.enum_address_types_impl(lp_enum_address_type_callback, guid_sp, context, flags)
            .ok()
    }

    fn enum_address_types_impl(
        &self,
        lp_enum_address_type_callback: extern "system" fn(*const GUID, *mut c_void, u32) -> BOOL,
        guid_sp: *mut GUID,
        context: *mut c_void,
        flags: u32,
    ) -> HRESULT {
        unsafe {
            self.dp_lobby
                .enum_address_types(lp_enum_address_type_callback, guid_sp, context, flags)
        }
    }

    pub fn enum_local_applications<T>(
        &self,
        lp_enum_local_app_callback: extern "system" fn(*const DPLAppInfo, *mut c_void, u32) -> BOOL,
        context: &mut T,
        flags: u32,
    ) -> Result<()> {
        unsafe {
            let context = std::mem::transmute::<*mut T, *mut c_void>(context);

            self.dp_lobby
                .enum_local_applications(lp_enum_local_app_callback, context, flags)
        }
        .ok()
    }

    pub fn run_application(
        &self,
        flags: u32,
        dw_app_id: *mut u32,
        lp_connection: *const DPLConnection,
        receive_event: *const c_void,
    ) -> Result<()> {
        unsafe {
            self.dp_lobby
                .run_application(flags, dw_app_id, lp_connection, receive_event)
        }
        .ok()
    }

    pub fn create_compound_address(
        &self,
        elements: *const DPCompoundAddressElement,
        element_count: u32,
        address: *mut c_void,
        address_size: *mut u32,
    ) -> Result<()> {
        unsafe {
            self.dp_lobby
                .create_compound_address(elements, element_count, address, address_size)
        }
        .ok()
    }
}
