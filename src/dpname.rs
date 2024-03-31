use windows::core::PCSTR;

#[repr(C)]
#[derive(Debug)]
pub struct DPName {
    pub size: u32,
    pub flags: u32,
    pub short_name: PCSTR,
    pub long_name: PCSTR,
}
