use std::ffi::c_void;
use windows::{
    core::*,
    Win32::{Foundation::*, System::Com::*},
};

#[derive(Debug)]
pub struct AppInfo {
    app_name: String,
    app_guid: GUID,
}

#[derive(Debug)]
pub struct ServiceProvider {
    sp_name: String,
    sp_guid: GUID,
}

static mut APPLICATIONS: Vec<AppInfo> = Vec::new();
static mut SERVICE_PROVIDERS: Vec<ServiceProvider> = Vec::new();

const CLSID_DIRECT_PLAY: GUID = GUID::from_u128(0xD1EB6D20_8923_11d0_9D97_00A0C90A43CB);
const CLSID_DIRECT_PLAY_LOBBY: GUID = GUID::from_u128(0x2FE8F810_B2A5_11d0_A787_0000F803ABFC);

#[repr(C)]
pub struct DPNAME {
    dwSize: u32,
    dwFlags: u32,
    lpszShortName: DPNAME_SHORTNAME,
    lpszLongName: DPNAME_LONGNAME,
}

#[repr(C)]
pub union DPNAME_SHORTNAME {
    lpszShortNameA: PSTR,
    lpszShortName: PWSTR,
}

#[repr(C)]
pub union DPNAME_LONGNAME {
    lpszLongNameA: PSTR,
    lpszLongName: PWSTR,
}

#[repr(C)]
pub struct DPLAPPINFO {
    dwSize: u32,
    guidApplication: GUID,
    appName: DPLAPPINFO_NAME,
}

#[repr(C)]
pub union DPLAPPINFO_NAME {
    lpszAppNameA: PSTR,
    lpszAppName: PWSTR,
}

#[interface("0AB1C531-4745-11D1-A7A1-0000F803ABFC")]
unsafe trait IDirectPlay4A: IUnknown {
    unsafe fn AddPlayerToGroup(&self, id_group: u32, id_player: u32) -> HRESULT;
    unsafe fn Close(&self) -> HRESULT;
    unsafe fn CreateGroup(
        &self,
        lpid_group: *mut u32,
        lp_group_name: *const DPNAME,
        lp_data: *const u8,
        data_size: u32,
        flags: u32,
    ) -> HRESULT;
    unsafe fn CreatePlayer(&self) -> HRESULT;
    unsafe fn DeletePlayerFromGroup(&self) -> HRESULT;
    unsafe fn DestroyGroup(&self) -> HRESULT;
    unsafe fn DestroyPlayer(&self) -> HRESULT;
    unsafe fn EnumGroupPlayers(&self) -> HRESULT;
    unsafe fn EnumGroups(&self) -> HRESULT;
    unsafe fn EnumPlayers(&self) -> HRESULT;
    unsafe fn EnumSessions(&self) -> HRESULT;
    unsafe fn GetCaps(&self) -> HRESULT;
    unsafe fn GetGroupData(&self) -> HRESULT;
    unsafe fn GetGroupName(&self) -> HRESULT;
    unsafe fn GetMessageCount(&self) -> HRESULT;
    unsafe fn GetPlayerAddress(&self) -> HRESULT;
    unsafe fn GetPlayerCaps(&self) -> HRESULT;
    unsafe fn GetPlayerData(&self) -> HRESULT;
    unsafe fn GetPlayerName(&self) -> HRESULT;
    unsafe fn GetSessionDesc(&self) -> HRESULT;
    unsafe fn Initialize(&self) -> HRESULT;
    unsafe fn Open(&self) -> HRESULT;
    unsafe fn Receive(&self) -> HRESULT;
    unsafe fn Send(&self) -> HRESULT;
    unsafe fn SetGroupData(&self) -> HRESULT;
    unsafe fn SetGroupName(&self) -> HRESULT;
    unsafe fn SetPlayerData(&self) -> HRESULT;
    unsafe fn SetPlayerName(&self) -> HRESULT;
    unsafe fn SetSessionDesc(&self) -> HRESULT;
    unsafe fn AddGroupToGroup(&self) -> HRESULT;
    unsafe fn CreateGroupInGroup(&self) -> HRESULT;
    unsafe fn DeleteGroupFromGroup(&self) -> HRESULT;
    unsafe fn EnumConnections(
        &self,
        lp_guid_application: *const GUID,
        lp_enum_callback: extern "system" fn(
            *const GUID,
            *const c_void,
            u32,
            *const DPNAME,
            u32,
            *const c_void,
        ) -> BOOL,
        lp_context: HWND,
        dw_flags: u32,
    ) -> HRESULT;
}

#[interface("2DB72491-652C-11d1-A7A8-0000F803ABFC")]
unsafe trait IDirectPlayLobby3A: IUnknown {
    unsafe fn Connect(
        &self,
        dw_flags: u32,
        lp_direct_play_2: *mut c_void,
        p_unk: *const c_void,
    ) -> HRESULT;
    unsafe fn CreateAddress(
        &self,
        guid_sp: *const GUID,
        guid_data_type: *const GUID,
        lp_data: *const c_void,
        dw_data_size: u32,
        lp_address: *mut c_void,
        lpdw_address_size: *mut u32,
    ) -> HRESULT;
    unsafe fn EnumAddress(
        &self,
        lp_enum_address_callback: *const c_void,
        lp_address: *const c_void,
        dw_address_size: u32,
        lp_context: *mut c_void,
    ) -> HRESULT;
    unsafe fn EnumAddressTypes(
        &self,
        lp_enum_address_type_callback: *const c_void,
        guid_sp: *const GUID,
        lp_context: *mut c_void,
        dw_flags: u32,
    ) -> HRESULT;
    unsafe fn EnumLocalApplications(
        &self,
        lp_enum_local_app_callback: extern "system" fn(
            *const DPLAPPINFO,
            *const c_void,
            u32,
        ) -> BOOL,
        lp_context: HWND,
        dw_flags: u32,
    ) -> HRESULT;
}

extern "system" fn enum_app(
    app_info: *const DPLAPPINFO,
    _context: *const c_void,
    _flags: u32,
) -> BOOL {
    unsafe {
        if let Some(app_info) = app_info.as_ref() {
            if let Ok(app_name) = app_info.appName.lpszAppNameA.to_string() {
                APPLICATIONS.push(AppInfo {
                    app_name,
                    app_guid: app_info.guidApplication,
                });
            }
        }
    }

    TRUE
}

extern "system" fn enum_sp(
    lp_guid_sp: *const GUID,
    lp_connection: *const c_void,
    dw_connection_size: u32,
    lp_name: *const DPNAME,
    dw_flags: u32,
    lp_context: *const c_void,
) -> BOOL {
    unsafe {
        if let Some(lp_name) = lp_name.as_ref() {
            if let Ok(sp_name) = lp_name.lpszShortName.lpszShortNameA.to_string() {
                if let Some(sp_guid) = lp_guid_sp.as_ref() {
                    SERVICE_PROVIDERS.push(ServiceProvider {
                        sp_guid: *sp_guid,
                        sp_name,
                    });
                }
            }
        }

    }

    TRUE
}

fn main() -> Result<()> {
    unsafe {
        CoInitialize(None).ok()?;

        let dp: IDirectPlay4A = CoCreateInstance(&CLSID_DIRECT_PLAY, None, CLSCTX_ALL)?;
        let dp_lobby: IDirectPlayLobby3A =
            CoCreateInstance(&CLSID_DIRECT_PLAY_LOBBY, None, CLSCTX_ALL)?;

        dp_lobby
            .EnumLocalApplications(enum_app, HWND::default(), 0)
            .ok()?;

        dp.EnumConnections(std::ptr::null(), enum_sp, HWND::default(), 0).ok()?;
    }

    Ok(())
}
