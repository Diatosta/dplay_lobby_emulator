use windows::core::{GUID, PCSTR};

#[repr(C)]
#[derive(Debug)]
pub struct DPSessionDesc2 {
    pub size: u32,
    pub flags: u32,
    pub guid_instance: GUID,
    pub guid_application: GUID,
    pub max_players: u32,
    pub current_players: u32,
    pub session_name: PCSTR,
    pub password: PCSTR,
    pub reserved_data_1: *mut u8,
    pub reserved_data_2: *mut u8,
    pub user_data_1: *mut u8,
    pub user_data_2: *mut u8,
    pub user_data_3: *mut u8,
    pub user_data_4: *mut u8,
}
