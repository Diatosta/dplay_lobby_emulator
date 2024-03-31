use windows::core::{GUID, PSTR};

#[repr(C)]
pub struct DPLAppInfo {
    pub size: u32,
    pub guid_application: GUID,
    pub app_name: PSTR,
}
