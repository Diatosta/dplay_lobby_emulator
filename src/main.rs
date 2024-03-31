mod direct_play_4;
mod direct_play_lobby_3;
mod dpcompound_address_element;
mod dplappinfo;
mod dplconnection;
mod dpname;
mod dpsession_desc2;

use crate::direct_play_4::{enum_connections, IDirectPlay4A};
use crate::direct_play_lobby_3::*;
use crate::dplappinfo::DPLAppInfo;
use crate::dpname::DPName;
use std::alloc::{alloc, Layout};

use crate::dpcompound_address_element::DPCompoundAddressElement;
use crate::dplconnection::{DPLConnection, DPOPEN_CREATE, DPOPEN_JOIN};
use crate::dpsession_desc2::DPSessionDesc2;
use std::ffi::c_void;
use windows::{
    core::*,
    Win32::{Foundation::*, System::Com::*},
};

const CLSID_DIRECT_PLAY_LOBBY: GUID = GUID::from_u128(0x2FE8F810_B2A5_11d0_A787_0000F803ABFC);
const CLSID_DIRECT_PLAY: GUID = GUID::from_u128(0xD1EB6D20_8923_11d0_9D97_00A0C90A43CB);
const DPAID_SERVICE_PROVIDER: GUID = GUID::from_u128(0x07D916C0_E0AF_11cf_9C4E_00A0C905425E);
const DPAID_INET: GUID = GUID::from_u128(0xC4A54DA0_E0AF_11cf_9C4E_00A0C905425E);
const SESSION_GUID: GUID = GUID::from_u128(0xA88F6634_2BA5_4986_99B2_A65CB5EC739C);

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

// Ideally these should be passed as context to the callback functions
// For now, we'll use static mutable variables
static mut APPLICATIONS: Vec<AppInfo> = Vec::new();
static mut SERVICE_PROVIDERS: Vec<ServiceProvider> = Vec::new();
static mut ADDRESS_TYPES: Vec<GUID> = Vec::new();

extern "system" fn enum_app(
    app_info: *const DPLAppInfo,
    _context: *const c_void,
    _flags: u32,
) -> BOOL {
    unsafe {
        if let Some(app_info) = app_info.as_ref() {
            if let Ok(app_name) = app_info.app_name.to_string() {
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
            if let Ok(sp_name) = lp_name.short_name.to_string() {
                // We only want TCP/IP service providers
                if !sp_name.contains("TCP/IP") {
                    return TRUE;
                }

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

extern "system" fn enum_addr_types(
    address_type: *const GUID,
    _context: *const c_void,
    _flags: u32,
) -> BOOL {
    unsafe {
        ADDRESS_TYPES.push(*address_type);
    }

    TRUE
}

fn create_addr(
    _hwnd: HWND,
    dp_lobby: &IDirectPlayLobby3A,
    service_provider_guid: *mut GUID,
    _address: *mut *mut c_void,
    _address_size: *mut u32,
) -> Result<(*mut c_void, u32)> {
    let mut address_elements: Vec<DPCompoundAddressElement> = Vec::new();

    unsafe {
        enum_address_types(
            &dp_lobby,
            enum_addr_types,
            service_provider_guid,
            std::ptr::null_mut(),
            0,
        )
        .ok()?;

        ADDRESS_TYPES.iter().for_each(|address_type| {
            println!("Address Type: {:?}", address_type);
        });
    }

    address_elements.push(DPCompoundAddressElement {
        guid_data_type: DPAID_SERVICE_PROVIDER,
        data_size: std::mem::size_of::<GUID>() as u32,
        data: service_provider_guid as *const c_void,
    });

    unsafe {
        // TODO: This should get the IP Address from somewhere, leave it empty for now
        ADDRESS_TYPES.iter().for_each(|_address_type| {
            address_elements.push(DPCompoundAddressElement {
                guid_data_type: DPAID_INET,
                data_size: 1,
                data: s!("").as_ptr() as *const c_void,
            });
        });

        address_elements.iter().for_each(|address_type| {
            println!("Address Type iter: {:?}", address_type);
        });
    }

    if address_elements.len() == 1 {
        return Err(Error::new(E_FAIL, "No address elements found"));
    }

    let mut address_size: u32 = 0;

    // See how large the buffer needs to be
    let result = unsafe {
        create_compound_address(
            &dp_lobby,
            address_elements.as_ptr(),
            address_elements.len() as u32,
            std::ptr::null_mut(),
            &mut address_size,
        )
    };

    // TODO: We should check if HRESULT is DPERR_BUFFERTOOSMALL, but I'm not sure how to yet
    // So we'll assume it went well for now

    let layout =
        Layout::from_size_align(address_size as usize, std::mem::align_of::<c_void>()).unwrap();

    let address = unsafe { alloc(layout) } as *mut c_void;

    let result = unsafe {
        create_compound_address(
            &dp_lobby,
            address_elements.as_ptr(),
            address_elements.len() as u32,
            address,
            &mut address_size,
        )
    };

    Ok((address, address_size))
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

        launch_direct_play_application(dp_lobby)?;
    }

    Ok(())
}

fn launch_direct_play_application(dp_lobby: IDirectPlayLobby3A) -> Result<()> {
    let mut guid_service_provider = unsafe { SERVICE_PROVIDERS.first().unwrap().sp_guid };

    let (address, address_size) = create_addr(
        HWND::default(),
        &dp_lobby,
        &mut guid_service_provider,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    )?;

    // TODO: Get the player name from somewhere
    let player_name = s!("");

    // TODO: Get if we're hosting or joining from somewhere
    let host = true;

    // TODO: Get the session name from somewhere
    let mut session_name: PCSTR = PCSTR::null();

    if host {
        session_name = s!("");
    }

    let app_guid = unsafe { APPLICATIONS.first().unwrap().app_guid };

    println!("App GUID: {:?}", app_guid);

    run_app(
        &dp_lobby,
        app_guid,
        guid_service_provider,
        address,
        address_size,
        session_name,
        player_name,
        host,
    )?;

    Ok(())
}

fn run_app(
    dp_lobby: &IDirectPlayLobby3A,
    application_guid: GUID,
    service_provider_guid: GUID,
    address: *mut c_void,
    address_size: u32,
    session_name: PCSTR,
    player_name: PCSTR,
    host_session: bool,
) -> Result<()> {
    let session_info = DPSessionDesc2 {
        size: std::mem::size_of::<DPSessionDesc2>() as u32,
        flags: 0,
        guid_instance: SESSION_GUID,
        // TODO: We just use the first application for now
        guid_application: application_guid,
        max_players: 0,
        current_players: 0,
        session_name: session_name,
        password: PCSTR::null(),
        reserved_data_1: std::ptr::null_mut(),
        reserved_data_2: std::ptr::null_mut(),
        user_data_1: std::ptr::null_mut(),
        user_data_2: std::ptr::null_mut(),
        user_data_3: std::ptr::null_mut(),
        user_data_4: std::ptr::null_mut(),
    };

    let player_name = DPName {
        size: std::mem::size_of::<DPName>() as u32,
        flags: 0,
        short_name: player_name,
        long_name: player_name,
    };

    let connect_info = DPLConnection {
        size: std::mem::size_of::<DPLConnection>() as u32,
        flags: match host_session {
            true => DPOPEN_CREATE,
            false => DPOPEN_JOIN,
        },
        session_desc: &session_info,
        player_name: &player_name,
        guid_sp: service_provider_guid,
        address,
        address_size,
    };

    let mut app_id: u32 = 0;

    unsafe {
        println!("Running application");
        println!("Session Info: {:?}", session_info);
        println!("Session Name: {}", session_info.session_name.to_string()?);
        println!("Player Name: {:?}", player_name);
        println!("Short name: {}", player_name.short_name.to_string()?);
        println!("Long name: {}", player_name.long_name.to_string()?);
        println!("Connect Info: {:?}", connect_info);

        run_application(dp_lobby, 0, &mut app_id, &connect_info, std::ptr::null()).ok()?;

        // We must wait for the app to start before we can close ours
        // But 5 seconds should be good enough for now
        // TODO: Wait for a success event from the app
        std::thread::sleep(std::time::Duration::from_secs(5));
    }

    Ok(())
}
