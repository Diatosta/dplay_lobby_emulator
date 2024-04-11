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
use std::cell::RefCell;

use crate::dpcompound_address_element::DPCompoundAddressElement;
use crate::dplconnection::{DPLConnection, DPOPEN_CREATE, DPOPEN_JOIN};
use crate::dpsession_desc2::DPSessionDesc2;
use std::ffi::c_void;
use std::rc::Rc;
use slint::{SharedString, VecModel};
use windows::{
    core::*,
    Win32::{Foundation::*, System::Com::*},
};

const CLSID_DIRECT_PLAY_LOBBY: GUID = GUID::from_u128(0x2FE8F810_B2A5_11d0_A787_0000F803ABFC);
const CLSID_DIRECT_PLAY: GUID = GUID::from_u128(0xD1EB6D20_8923_11d0_9D97_00A0C90A43CB);
const DPAID_SERVICE_PROVIDER: GUID = GUID::from_u128(0x07D916C0_E0AF_11cf_9C4E_00A0C905425E);
const DPAID_INET: GUID = GUID::from_u128(0xC4A54DA0_E0AF_11cf_9C4E_00A0C905425E);
const SESSION_GUID: GUID = GUID::from_u128(0xA88F6634_2BA5_4986_99B2_A65CB5EC739C);

#[derive(Debug, Clone)]
pub struct AppInfo {
    app_name: String,
    app_guid: GUID,
}

#[derive(Debug, Clone)]
pub struct ServiceProvider {
    sp_name: String,
    sp_guid: GUID,
}

pub struct SessionInfo {
    selected_app: Option<AppInfo>,
    selected_service_provider: Option<ServiceProvider>,
    session_name: String,
    player_name: String,
    host_session: bool,
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

slint::include_modules!();
fn main() -> Result<()> {
    let session_info: Rc<RefCell<SessionInfo>> = Rc::new(RefCell::new(SessionInfo {
        selected_app: None,
        selected_service_provider: None,
        session_name: String::new(),
        player_name: String::new(),
        host_session: true,
    }));

    unsafe {
        let app_window = AppWindow::new().unwrap();

        CoInitialize(None).ok()?;

        let dp: IDirectPlay4A = CoCreateInstance(&CLSID_DIRECT_PLAY, None, CLSCTX_ALL)?;
        let dp_lobby: Rc<RefCell<IDirectPlayLobby3A>> =
            Rc::new(RefCell::new(CoCreateInstance(&CLSID_DIRECT_PLAY_LOBBY, None, CLSCTX_ALL)?));

        enum_local_applications(&dp_lobby.borrow(), enum_app, HWND::default(), 0).ok()?;

        enum_connections(&dp, std::ptr::null(), enum_sp, HWND::default(), 0).ok()?;

        // Create a String array from APPLICATIONS app_name
        let app_names: Vec<SharedString> = APPLICATIONS.iter().map(|app| app.app_name.clone().into()).collect();
        let first_app_name = app_names.get(0).unwrap().clone();
        let app_names = Rc::new(VecModel::from(app_names));
        app_window.set_application_names(app_names.clone().into());
        app_window.set_selected_application_name(first_app_name.clone());
        // TODO: Change this to a dedicated method
        session_info.borrow_mut().selected_app = get_selected_application(app_window.get_selected_application_name().as_str());

        // Create a String array from SERVICE_PROVIDERS sp_name
        let sp_names: Vec<SharedString> = SERVICE_PROVIDERS.iter().map(|sp| sp.sp_name.clone().into()).collect();
        let first_sp_name = sp_names.get(0).unwrap().clone();
        let sp_names = Rc::new(VecModel::from(sp_names));
        app_window.set_service_provider_names(sp_names.clone().into());

        app_window.set_selected_service_provider_name(first_sp_name.clone());
        app_window.set_address_type(set_address_type(first_sp_name.as_str()));

        // TODO: Change this to a dedicated method
        session_info.borrow_mut().selected_service_provider = get_selected_service_provider(app_window.get_selected_service_provider_name().as_str());

        let session_info_app_weak = session_info.clone();

        app_window.on_change_selected_application(move |value| {
            session_info_app_weak.borrow_mut().selected_app = get_selected_application(&value);
        });

        let app_window_address_type_weak = app_window.as_weak();
        let session_info_service_provider_weak = session_info.clone();

        app_window.on_change_selected_service_provider(move |value| {
            let app_window = app_window_address_type_weak.unwrap();
            session_info_service_provider_weak.borrow_mut().selected_service_provider = get_selected_service_provider(&value);
            app_window.set_address_type(set_address_type(&value));
        });

        let app_window_weak = app_window.as_weak();

        app_window.on_click_run_application(move || {
            let app_window = app_window_weak.unwrap();
            let mut session_info = session_info.borrow_mut();

            session_info.player_name = app_window.get_player_name().parse().unwrap();
            session_info.session_name = app_window.get_session_name().parse().unwrap();
            session_info.host_session = app_window.get_is_host();

            let result = launch_direct_play_application(&dp_lobby.borrow(), &session_info, &app_window);
            if let Err(error) = result {
                app_window.set_status("Failed to launch application".into());

                println!("Error: {:?}", error);
            }
        });

        app_window.run().unwrap();
    }

    Ok(())
}

fn set_address_type(sp_name: &str) -> slint_generatedAppWindow::AddressType {
    match sp_name.to_lowercase() {
        x if x.contains("tcp") => slint_generatedAppWindow::AddressType::TCPIP,
        x if x.contains("modem") => slint_generatedAppWindow::AddressType::Modem,
        x if x.contains("serial") => slint_generatedAppWindow::AddressType::Serial,
        _ => slint_generatedAppWindow::AddressType::None,
    }
}

fn get_selected_application(app_name: &str) -> Option<AppInfo> {
    unsafe {
        APPLICATIONS.iter().find(|app| app.app_name == app_name).cloned()
    }
}

fn get_selected_service_provider(sp_name: &str) -> Option<ServiceProvider> {
    unsafe {
        SERVICE_PROVIDERS.iter().find(|sp| sp.sp_name == sp_name).cloned()
    }
}

fn launch_direct_play_application(dp_lobby: &IDirectPlayLobby3A, session_info: &SessionInfo, app_window: &AppWindow) -> Result<()> {
    let mut guid_service_provider = session_info.selected_service_provider.clone().ok_or_else(|| Error::new(E_FAIL, "No service provider selected"))?.sp_guid;
    let app_guid = session_info.selected_app.clone().ok_or_else(|| Error::new(E_FAIL, "No application selected"))?.app_guid;

    app_window.set_status("Launching application...".into());

    let (address, address_size) = create_addr(
        HWND::default(),
        &dp_lobby,
        &mut guid_service_provider,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    )?;

    run_app(
        &dp_lobby,
        app_guid,
        guid_service_provider,
        address,
        address_size,
        PCSTR(format!("{}\0", session_info.session_name).as_ptr()),
        PCSTR(format!("{}\0", session_info.player_name).as_ptr()),
        session_info.host_session,
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
        guid_application: application_guid,
        max_players: 0,
        current_players: 0,
        session_name,
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

    run_application(dp_lobby, 0, &mut app_id, &connect_info, std::ptr::null())?;

    Ok(())
}
