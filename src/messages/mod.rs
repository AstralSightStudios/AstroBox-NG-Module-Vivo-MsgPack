pub mod bind;
pub mod device_info;

pub const fn response_cid(cid: u8) -> u8 {
    cid | 0x80
}

pub const fn request_cid(cid: u8) -> u8 {
    cid & 0x7f
}

pub const fn is_response_cid(cid: u8) -> bool {
    (cid & 0x80) == 0x80
}
