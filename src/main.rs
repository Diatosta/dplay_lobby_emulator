#![windows_subsystem = "windows"]

mod direct_play_4;
mod direct_play_lobby_3;
mod dpcompound_address_element;
mod dplappinfo;
mod dplconnection;
mod dpname;
mod dpsession_desc2;

use crate::direct_play_4::*;
use crate::direct_play_lobby_3::*;
use crate::dplappinfo::DPLAppInfo;
use crate::dpname::DPName;
use std::alloc::{alloc, Layout};
use std::cell::RefCell;
use std::ffi::CString;

use crate::dpcompound_address_element::DPCompoundAddressElement;
use crate::dplconnection::{DPLConnection, DPOPEN_CREATE, DPOPEN_JOIN};
use crate::dpsession_desc2::DPSessionDesc2;
use slint::{SharedString, VecModel};
use std::ffi::c_void;
use std::rc::Rc;
use windows::{
    core::*,
    Win32::{Foundation::*, System::Com::*},
};

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

slint::include_modules!();
fn main() -> Result<()> {
    let session_info: Rc<RefCell<SessionInfo>> = Rc::new(RefCell::new(SessionInfo {
        selected_app: None,
        selected_service_provider: None,
        session_name: String::new(),
        player_name: String::new(),
        host_session: true,
    }));

    let mut applications: Vec<AppInfo> = Vec::new();
    let mut service_providers: Vec<ServiceProvider> = Vec::new();
    let mut address_types: Vec<GUID> = Vec::new();

    unsafe {
        let app_window = AppWindow::new().unwrap();

        // We must initialize COM before calling any COM functions
        CoInitialize(None).ok()?;

        let dp: DirectPlay4A = DirectPlay4A::new()?;
        let dp_lobby: Rc<RefCell<DirectPlayLobby3A>> =
            Rc::new(RefCell::new(DirectPlayLobby3A::new()?));

        dp_lobby
            .borrow()
            .enum_local_applications(enum_app, &mut applications, 0)?;

        dp.enum_connections(std::ptr::null(), enum_sp, &mut service_providers, 0)?;

        let app_names: Vec<SharedString> = applications
            .iter()
            .map(|app| app.app_name.clone().into())
            .collect();

        let first_app_name = app_names.get(0).cloned().unwrap_or_default();
        let app_names = Rc::new(VecModel::from(app_names));
        app_window.set_application_names(app_names.clone().into());
        app_window.set_selected_application_name(first_app_name.clone());
        session_info.borrow_mut().selected_app = get_selected_application(
            app_window.get_selected_application_name().as_str(),
            &mut applications,
        );

        let sp_names: Vec<SharedString> = service_providers
            .iter()
            .map(|sp| sp.sp_name.clone().into())
            .collect();

        let first_sp_name = sp_names.get(0).cloned().unwrap_or_default();
        let sp_names = Rc::new(VecModel::from(sp_names));
        app_window.set_service_provider_names(sp_names.clone().into());

        app_window.set_selected_service_provider_name(first_sp_name.clone());
        app_window.set_address_type(set_address_type(first_sp_name.as_str()));

        session_info.borrow_mut().selected_service_provider = get_selected_service_provider(
            app_window.get_selected_service_provider_name().as_str(),
            &mut service_providers,
        );

        {
            let session_info = session_info.clone();

            app_window.on_change_selected_application(move |value| {
                session_info.borrow_mut().selected_app =
                    get_selected_application(&value, &mut applications);
            });
        }

        {
            let session_info = session_info.clone();
            let app_window_weak = app_window.as_weak();

            app_window.on_change_selected_service_provider(move |value| {
                let app_window = app_window_weak.unwrap();
                session_info.borrow_mut().selected_service_provider =
                    get_selected_service_provider(&value, &mut service_providers);
                app_window.set_address_type(set_address_type(&value));
            });
        }

        {
            let app_window_weak = app_window.as_weak();

            app_window.on_click_run_application(move || {
                let app_window = app_window_weak.unwrap();
                let mut session_info = session_info.borrow_mut();

                session_info.player_name = app_window.get_player_name().to_string();
                session_info.session_name = app_window.get_session_name().to_string();
                session_info.host_session = app_window.get_is_host();

                let result = launch_direct_play_application(
                    &dp_lobby.borrow(),
                    &session_info,
                    &app_window,
                    &mut address_types,
                );
                if let Err(error) = result {
                    app_window.set_status("Failed to launch application".into());

                    println!("Error: {:?}", error);
                }
            });
        }

        {
            let app_window_weak = app_window.as_weak();

            app_window.on_close_app(move || {
                CoUninitialize();
                let _ = app_window_weak.unwrap().hide();
            });
        }

        app_window.run().unwrap();
    }

    Ok(())
}

extern "system" fn enum_app(
    app_info: *const DPLAppInfo,
    context: *mut c_void,
    _flags: u32,
) -> BOOL {
    let app_names: &mut Vec<AppInfo> = unsafe {
        let context_raw = context as *mut Vec<AppInfo>;

        context_raw
            .as_mut()
            .expect("Failed to get mutable reference to app_names")
    };

    unsafe {
        if let Some(app_info) = app_info.as_ref() {
            if let Ok(app_name) = app_info.app_name.to_string() {
                app_names.push(AppInfo {
                    app_name,
                    app_guid: app_info.guid_application,
                });
            }
        }
    }

    TRUE
}

extern "system" fn enum_sp(
    guid_sp: *const GUID,
    _lp_connection: *const c_void,
    _dw_connection_size: u32,
    name: *const DPName,
    _dw_flags: u32,
    context: *mut c_void,
) -> BOOL {
    let service_providers: &mut Vec<ServiceProvider> = unsafe {
        let context_raw = context as *mut Vec<ServiceProvider>;

        context_raw
            .as_mut()
            .expect("Failed to get mutable reference to service_providers")
    };

    unsafe {
        if let Some(name) = name.as_ref() {
            if let Ok(sp_name) = name.short_name.to_string() {
                if let Some(sp_guid) = guid_sp.as_ref() {
                    service_providers.push(ServiceProvider {
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
    context: *mut c_void,
    _flags: u32,
) -> BOOL {
    let address_types: &mut Vec<GUID> = unsafe {
        let context_raw = context as *mut Vec<GUID>;

        context_raw
            .as_mut()
            .expect("Failed to get mutable reference to address_types")
    };

    unsafe {
        address_types.push(*address_type);
    }

    TRUE
}

fn create_addr(
    dp_lobby: &DirectPlayLobby3A,
    service_provider_guid: &mut GUID,
    address_types: &mut Vec<GUID>,
    app_window: &AppWindow,
) -> Result<(*mut c_void, u32)> {
    let mut address_elements: Vec<DPCompoundAddressElement> = Vec::new();

    dp_lobby.enum_address_types::<Vec<GUID>>(
        enum_addr_types,
        service_provider_guid,
        address_types,
        0,
    )?;

    address_elements.push(DPCompoundAddressElement {
        guid_data_type: DPAID_SERVICE_PROVIDER,
        data_size: std::mem::size_of::<GUID>() as u32,
        data: unsafe { std::mem::transmute::<&mut GUID, *const c_void>(service_provider_guid) },
    });

    let guid_address_type = address_types
        .get(0)
        .ok_or(Error::new(E_FAIL, "No address types found"))?;

    match guid_address_type {
        &DPAID_INET => {
            if let Ok(data) = CString::new(app_window.get_ip_address().as_str()) {
                address_elements.push(DPCompoundAddressElement {
                    guid_data_type: DPAID_INET,
                    data_size: data.as_bytes().len() as u32 + 1,
                    data: data.as_ptr() as *const c_void,
                });
            } else {
                return Err(Error::new(E_FAIL, "Failed to get IP address"));
            }
        }
        &DPAID_PHONE => {
            if let Ok(data) = CString::new(app_window.get_phone_number().as_str()) {
                address_elements.push(DPCompoundAddressElement {
                    guid_data_type: DPAID_PHONE,
                    data_size: data.as_bytes().len() as u32 + 1,
                    data: data.as_ptr() as *const c_void,
                });
            } else {
                return Err(Error::new(E_FAIL, "Failed to get phone number"));
            }
        }
        _ => {}
    }

    if address_elements.len() == 1 {
        return Err(Error::new(E_FAIL, "No address elements found"));
    }

    let mut address_size: u32 = 0;

    // See how large the buffer needs to be
    if {
        dp_lobby.create_compound_address(
            address_elements.as_ptr(),
            address_elements.len() as u32,
            std::ptr::null_mut(),
            &mut address_size,
        )
    }
    .is_ok()
    {
        // This call shouldn't succeed
        return Err(Error::new(E_FAIL, "Failed to get address size"));
    }

    let layout =
        Layout::from_size_align(address_size as usize, std::mem::align_of::<c_void>()).unwrap();

    let address = unsafe { alloc(layout) } as *mut c_void;

    dp_lobby.create_compound_address(
        address_elements.as_ptr(),
        address_elements.len() as u32,
        address,
        &mut address_size,
    )?;

    Ok((address, address_size))
}

fn set_address_type(sp_name: &str) -> slint_generatedAppWindow::AddressType {
    match sp_name.to_lowercase() {
        x if x.contains("tcp") => slint_generatedAppWindow::AddressType::TCPIP,
        x if x.contains("modem") => slint_generatedAppWindow::AddressType::Modem,
        x if x.contains("serial") => slint_generatedAppWindow::AddressType::Serial,
        _ => slint_generatedAppWindow::AddressType::None,
    }
}

fn get_selected_application(app_name: &str, applications: &mut Vec<AppInfo>) -> Option<AppInfo> {
    applications
        .iter()
        .find(|app| app.app_name == app_name)
        .cloned()
}

fn get_selected_service_provider(
    sp_name: &str,
    service_providers: &mut Vec<ServiceProvider>,
) -> Option<ServiceProvider> {
    service_providers
        .iter()
        .find(|sp| sp.sp_name == sp_name)
        .cloned()
}

fn launch_direct_play_application(
    dp_lobby: &DirectPlayLobby3A,
    session_info: &SessionInfo,
    app_window: &AppWindow,
    address_types: &mut Vec<GUID>,
) -> Result<()> {
    let mut guid_service_provider = session_info
        .selected_service_provider
        .clone()
        .ok_or_else(|| Error::new(E_FAIL, "No service provider selected"))?
        .sp_guid;
    let app_guid = session_info
        .selected_app
        .clone()
        .ok_or_else(|| Error::new(E_FAIL, "No application selected"))?
        .app_guid;

    app_window.set_status("Launching application...".into());

    let (address, address_size) = create_addr(
        &dp_lobby,
        &mut guid_service_provider,
        address_types,
        &app_window,
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

    app_window.set_status("Launch successful".into());

    Ok(())
}

fn run_app(
    dp_lobby: &DirectPlayLobby3A,
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

    dp_lobby.run_application(0, &mut app_id, &connect_info, std::ptr::null())?;

    Ok(())
}
