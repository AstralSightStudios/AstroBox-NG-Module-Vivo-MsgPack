use crate::Result;
use crate::messages::response_cid;
use crate::msgpack::{MsgpackReader, write_array_len, write_bin, write_bool, write_i32, write_str};

pub const BID_DEVICE_INFO: u8 = 25;
pub const CID_FIRST_SYNC: u8 = 1;
pub const CID_DEVICE_INFO: u8 = 2;
pub const CID_WATCH_BID_VERSION: u8 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BidVersion {
    pub bid: i32,
    pub version: i32,
}

impl BidVersion {
    pub fn payload(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_i32(&mut out, self.bid);
        write_i32(&mut out, self.version);
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureItem {
    pub key: i32,
    pub value: String,
}

impl FeatureItem {
    pub fn payload(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        write_i32(&mut out, self.key);
        write_str(&mut out, &self.value)?;
        Ok(out)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchBidVersionRequest {
    pub sync_bid_versions: bool,
    pub sync_features: bool,
    pub phone_bid_version: i32,
    pub bid_versions: Vec<BidVersion>,
    pub features: Vec<FeatureItem>,
}

impl WatchBidVersionRequest {
    pub fn payload(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        write_bool(&mut out, self.sync_bid_versions);
        write_bool(&mut out, self.sync_features);
        write_i32(&mut out, self.phone_bid_version);
        write_array_len(&mut out, self.bid_versions.len())?;
        for bid_version in &self.bid_versions {
            write_bin(&mut out, &bid_version.payload())?;
        }
        write_array_len(&mut out, self.features.len())?;
        for feature in &self.features {
            write_bin(&mut out, &feature.payload()?)?;
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchBidVersionResponse {
    pub code: i32,
    pub phone_bid_version: i32,
    pub bid_versions: Vec<BidVersion>,
    pub features: Vec<FeatureItem>,
}

impl WatchBidVersionResponse {
    pub const CID: u8 = response_cid(CID_WATCH_BID_VERSION);

    pub fn decode(payload: &[u8]) -> Result<Self> {
        let mut reader = MsgpackReader::new(payload);
        let code = reader.read_i32()?;
        let phone_bid_version = reader.read_i32()?;
        let bid_count = reader.read_array_len()?;
        let mut bid_versions = Vec::with_capacity(bid_count);
        for _ in 0..bid_count {
            let nested = reader.read_bin()?;
            let mut nested = MsgpackReader::new(&nested);
            bid_versions.push(BidVersion {
                bid: nested.read_i32()?,
                version: nested.read_i32()?,
            });
        }

        let mut features = Vec::new();
        if reader.has_next() {
            let feature_count = reader.read_array_len()?;
            features.reserve(feature_count);
            for _ in 0..feature_count {
                let nested = reader.read_bin()?;
                let mut nested = MsgpackReader::new(&nested);
                features.push(FeatureItem {
                    key: nested.read_i32()?,
                    value: nested.read_str()?,
                });
            }
        }

        Ok(Self {
            code,
            phone_bid_version,
            bid_versions,
            features,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceInfoRequest {
    pub unix_time_sec: i32,
    pub gmt_timezone: i32,
    pub os_type: i32,
    pub millis: i32,
    pub timezone_offset_sec: Option<i32>,
}

impl DeviceInfoRequest {
    pub fn payload(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_i32(&mut out, self.unix_time_sec);
        write_i32(&mut out, self.gmt_timezone);
        write_i32(&mut out, self.os_type);
        write_i32(&mut out, self.millis);
        if let Some(offset) = self.timezone_offset_sec {
            write_i32(&mut out, offset);
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchFirstSyncResp {
    pub code: i32,
    pub battery: i32,
    pub total_storage: i64,
    pub free_storage: i64,
    pub device_name: String,
    pub version: String,
    pub sn: String,
    pub mac: String,
    pub model: String,
    pub version_type: i32,
    pub ble_mac: String,
    pub product_id: i32,
}

impl WatchFirstSyncResp {
    pub const CID: u8 = response_cid(CID_FIRST_SYNC);

    pub fn decode_prefix(payload: &[u8]) -> Result<Self> {
        let mut reader = MsgpackReader::new(payload);
        Ok(Self {
            code: reader.read_i32()?,
            battery: reader.read_i32()?,
            total_storage: reader.read_i64()?,
            free_storage: reader.read_i64()?,
            device_name: reader.read_str()?,
            version: reader.read_str()?,
            sn: reader.read_str()?,
            mac: reader.read_str()?,
            model: reader.read_str()?,
            version_type: reader.read_i32()?,
            ble_mac: reader.read_str()?,
            product_id: reader.read_i32()?,
        })
    }
}
