use crate::messages::response_cid;
use crate::msgpack::{MsgpackReader, write_bin, write_bool, write_i32, write_i64, write_str};
use crate::{MsgpackError, Result};

pub const BID_BIND: u8 = 15;
pub const CID_AUTH: u8 = 1;
pub const CID_INIT: u8 = 2;
pub const CID_BIND: u8 = 3;
pub const CID_RESET: u8 = 5;
pub const CID_CONNECT_CONFIRM: u8 = 6;
pub const CID_HFP_CONNECT: u8 = 18;
pub const DEFAULT_BID_VERSION: i32 = 11;
pub const MAGIC_PHONE_PLACEHOLDER: &str = "***********";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindAuthRequest {
    pub open_id: String,
    pub random: u16,
    pub aes_sign: Vec<u8>,
    pub bid_version: i32,
    pub magic_phone: String,
}

impl BindAuthRequest {
    pub fn new(
        open_id: impl Into<String>,
        random: u16,
        aes_sign: Vec<u8>,
        magic_phone: Option<String>,
    ) -> Self {
        Self {
            open_id: open_id.into(),
            random,
            aes_sign,
            bid_version: DEFAULT_BID_VERSION,
            magic_phone: magic_phone
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| MAGIC_PHONE_PLACEHOLDER.to_string()),
        }
    }

    pub fn payload(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        write_str(&mut out, &self.open_id)?;
        write_i32(&mut out, self.random as i32);
        write_bin(&mut out, &self.aes_sign)?;
        write_i32(&mut out, self.bid_version);
        write_str(&mut out, &self.magic_phone)?;
        Ok(out)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindAuthResponse {
    pub code: i32,
    pub sn: String,
    pub random: i32,
    pub aes_sign: Vec<u8>,
    pub bid_version: i64,
    pub seq: i64,
}

impl BindAuthResponse {
    pub const CID: u8 = response_cid(CID_AUTH);

    pub fn decode(payload: &[u8]) -> Result<Self> {
        let mut reader = MsgpackReader::new(payload);
        Ok(Self {
            code: reader.read_i32()?,
            sn: reader.read_str()?,
            random: reader.read_i32()?,
            aes_sign: reader.read_bin()?,
            bid_version: reader.read_i64()?,
            seq: reader.read_i64()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindInitRequest {
    pub os: i32,
    pub is_bind: bool,
    pub version: String,
    pub module: String,
    pub phone_device_id: String,
    pub seq: i64,
}

impl BindInitRequest {
    pub fn payload(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        write_i32(&mut out, self.os);
        write_bool(&mut out, self.is_bind);
        write_str(&mut out, &self.version)?;
        write_str(&mut out, &self.module)?;
        write_str(&mut out, &self.phone_device_id)?;
        write_i64(&mut out, self.seq);
        Ok(out)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum WatchBindState {
    Init = 0,
    Bind = 1,
    Mid = 2,
    Unknown = -1,
}

impl From<i32> for WatchBindState {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Init,
            1 => Self::Bind,
            2 => Self::Mid,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindInitResponse {
    pub code: i32,
    pub open_id: String,
    pub version: String,
    pub pack_size: i32,
    pub state: WatchBindState,
    pub phone_device_id: String,
    pub last_app_version: String,
    pub unused: i32,
    pub seq: i64,
    pub is_phone_bt_mac_empty: Option<bool>,
    pub max_data_length: Option<i32>,
    pub magic_phone: Option<String>,
}

impl BindInitResponse {
    pub const CID: u8 = response_cid(CID_INIT);

    pub fn decode(payload: &[u8]) -> Result<Self> {
        let mut reader = MsgpackReader::new(payload);
        let code = reader.read_i32()?;
        let open_id = reader.read_str()?;
        let version = reader.read_str()?;
        let pack_size = reader.read_i32()?;
        let state = WatchBindState::from(reader.read_i32()?);
        let phone_device_id = reader.read_str()?;
        let last_app_version = reader.read_str()?;
        let unused = reader.read_i32()?;
        let seq = reader.read_i64()?;
        let is_phone_bt_mac_empty = if reader.has_next() {
            Some(reader.read_bool()?)
        } else {
            None
        };
        let max_data_length = if reader.has_next() {
            Some(reader.read_i32()?)
        } else {
            None
        };
        let magic_phone = if reader.has_next() {
            Some(reader.read_str()?)
        } else {
            None
        };

        Ok(Self {
            code,
            open_id,
            version,
            pack_size,
            state,
            phone_device_id,
            last_app_version,
            unused,
            seq,
            is_phone_bt_mac_empty,
            max_data_length,
            magic_phone,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindBindRequest {
    pub confirm: bool,
}

impl BindBindRequest {
    pub fn payload(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_bool(&mut out, self.confirm);
        out
    }
}

pub fn empty_payload() -> Vec<u8> {
    Vec::new()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindConnectConfirmRequest {
    pub seq: i64,
    pub has_backup_list: bool,
}

impl BindConnectConfirmRequest {
    pub fn payload(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_i64(&mut out, self.seq);
        write_bool(&mut out, self.has_backup_list);
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResponseCode {
    pub code: i32,
}

impl ResponseCode {
    pub fn decode(payload: &[u8]) -> Result<Self> {
        let mut reader = MsgpackReader::new(payload);
        Ok(Self {
            code: reader.read_i32()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindConnectConfirmResponse {
    pub code: i32,
    pub seq: Option<i64>,
}

impl BindConnectConfirmResponse {
    pub const CID: u8 = response_cid(CID_CONNECT_CONFIRM);

    pub fn decode(payload: &[u8]) -> Result<Self> {
        let mut reader = MsgpackReader::new(payload);
        let code = reader.read_i32()?;
        let seq = if reader.has_next() {
            Some(reader.read_i64()?)
        } else {
            None
        };
        Ok(Self { code, seq })
    }
}

pub fn require_success(code: i32) -> Result<()> {
    if code == 0 {
        Ok(())
    } else {
        Err(MsgpackError::ResponseCode(code))
    }
}
