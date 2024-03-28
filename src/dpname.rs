use windows::core::{PSTR, PWSTR};

#[repr(C)]
pub struct DPName {
    pub size: u32,
    pub flags: u32,
    pub short_name: DPNameShortName,
    pub long_name: DPNameLongName,
}

#[repr(C)]
pub union DPNameShortName {
    pub short_name_a: PSTR,
    pub short_name: PWSTR,
}

#[repr(C)]
pub union DPNameLongName {
    pub long_name_a: PSTR,
    pub long_name: PWSTR,
}