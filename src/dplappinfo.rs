use windows::core::{PSTR, PWSTR, GUID};

#[repr(C)]
pub struct DPLAppInfo {
    pub size: u32,
    pub guid_application: GUID,
    pub app_name: DPLAppInfoName,
}

#[repr(C)]
pub union DPLAppInfoName {
    pub app_name_a: PSTR,
    pub app_name: PWSTR,
}