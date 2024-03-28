mod direct_play_4;
mod direct_play_lobby_3;
mod dplappinfo;
mod dpname;

use crate::direct_play_4::{IDirectPlay4A, enum_connections};
use crate::direct_play_lobby_3::{IDirectPlayLobby3A, enum_local_applications};
use crate::dplappinfo::DPLAppInfo;
use crate::dpname::DPName;

use std::ffi::c_void;
use windows::{
    core::*,
    Win32::{Foundation::*, System::Com::*},
};

const CLSID_DIRECT_PLAY_LOBBY: GUID = GUID::from_u128(0x2FE8F810_B2A5_11d0_A787_0000F803ABFC);
const CLSID_DIRECT_PLAY: GUID = GUID::from_u128(0xD1EB6D20_8923_11d0_9D97_00A0C90A43CB);

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

extern "system" fn enum_app(
    app_info: *const DPLAppInfo,
    _context: *const c_void,
    _flags: u32,
) -> BOOL {
    unsafe {
        if let Some(app_info) = app_info.as_ref() {
            if let Ok(app_name) = app_info.app_name.app_name_a.to_string() {
                APPLICATIONS.push(AppInfo {
                    app_name,
                    app_guid: app_info.guid_application,
                });
            }
        }
    }

    TRUE
}

extern "system" fn enum_sp(
    lp_guid_sp: *const GUID,
    _lp_connection: *const c_void,
    _dw_connection_size: u32,
    lp_name: *const DPName,
    _dw_flags: u32,
    _lp_context: *const c_void,
) -> BOOL {
    unsafe {
        if let Some(lp_name) = lp_name.as_ref() {
            if let Ok(sp_name) = lp_name.short_name.short_name_a.to_string() {
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

        enum_local_applications(&dp_lobby, enum_app, HWND::default(), 0).ok()?;

        enum_connections(&dp, std::ptr::null(), enum_sp, HWND::default(), 0).ok()?;

        APPLICATIONS.iter().for_each(|app| {
            println!("App Name: {:?}, App GUID: {:?}", app.app_name, app.app_guid);
        });

        SERVICE_PROVIDERS.iter().for_each(|sp| {
            println!("SP Name: {:?}, SP GUID: {:?}", sp.sp_name, sp.sp_guid);
        });
    }

    Ok(())
}
