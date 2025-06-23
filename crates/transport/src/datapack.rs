use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};
use ulid::Ulid;

use crate::{channel::Channel, util::get_chunk};

const UNASSIGNED_ID: Ulid = Ulid::from_bytes([0; 16]);

pub struct RawDataPack {
    data_len: u32,
    data:     Bytes,
}

impl RawDataPack {
    const fn new(data: Bytes) -> Self {
        Self {
            data_len: data.len() as u32,
            data,
        }
    }
}

pub struct RawDataPackCodec {
    data_len:  Option<u32>,
    de_buffer: BytesMut,
    en_buffer: BytesMut,
}

impl RawDataPackCodec {
    #[must_use]
    pub fn new() -> Self {
        Self {
            data_len:  None,
            de_buffer: BytesMut::new(),
            en_buffer: BytesMut::new(),
        }
    }
}

impl Default for RawDataPackCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Encoder<RawDataPack> for RawDataPackCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: RawDataPack, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.en_buffer.put_u32(item.data_len);
        self.en_buffer.put(item.data);
        while let Some(bytes) = get_chunk(&mut self.en_buffer) {
            dst.put(bytes);
        }
        Ok(())
    }
}

impl Decoder for RawDataPackCodec {
    type Error = std::io::Error;
    type Item = RawDataPack;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.de_buffer.put(src.split());
        if self.de_buffer.len() < 4 && self.data_len.is_none() {
            return Ok(None);
        } else if self.data_len.is_none() {
            self.data_len = Some(self.de_buffer.get_u32());
        }
        let Some(data_len) = self.data_len else {
            return Ok(None);
        };
        if self.de_buffer.len() < (data_len as usize) {
            return Ok(None);
        }
        let data = self.de_buffer.split_to(data_len as usize);
        self.data_len = None;
        Ok(Some(Self::Item {
            data_len,
            data: data.into(),
        }))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DataPack {
    #[serde(flatten)]
    pub header:      Header,
    pub correlation: Ulid,
    pub channel:     Option<Channel>,
    #[serde(flatten)]
    pub result:      DataResult,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Header {
    Request { path: String },
    Response,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestDataPack {
    pub path:    String,
    correlation: Ulid,
    pub channel: Option<Channel>,
    pub payload: rmpv::Value,
}

impl Default for RequestDataPack {
    fn default() -> Self {
        Self {
            path:        String::new(),
            correlation: Ulid::new(),
            channel:     None,
            payload:     rmpv::Value::Nil,
        }
    }
}

impl RequestDataPack {
    #[must_use]
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    #[must_use]
    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = Some(channel);
        self
    }

    #[must_use]
    pub fn payload_value(mut self, payload: impl Into<rmpv::Value>) -> Self {
        self.payload = payload.into();
        self
    }

    #[must_use]
    pub fn payload<S: Serialize>(mut self, payload: S) -> Self {
        self.payload = rmpv::ext::to_value(payload).unwrap_or(rmpv::Value::Nil);
        self
    }

    #[must_use]
    pub const fn correlation(&self) -> Ulid {
        self.correlation
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseDataPack {
    pub correlation: Ulid,
    pub channel:     Option<Channel>,
    #[serde(flatten)]
    pub result:      DataResult,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DataResult {
    #[serde(rename = "payload")]
    Payload(rmpv::Value),
    #[serde(rename = "error")]
    Error(String),
}

impl From<DataResult> for Result<rmpv::Value, String> {
    fn from(value: DataResult) -> Self {
        match value {
            DataResult::Payload(v) => Ok(v),
            DataResult::Error(e) => Err(e),
        }
    }
}

impl Default for ResponseDataPack {
    fn default() -> Self {
        Self {
            correlation: UNASSIGNED_ID,
            channel:     None,
            result:      DataResult::Payload(rmpv::Value::Nil),
        }
    }
}

impl ResponseDataPack {
    pub const fn correlate(&mut self, correlation: Ulid) {
        self.correlation = correlation;
    }

    #[must_use]
    pub fn payload_value(mut self, payload: impl Into<rmpv::Value>) -> Self {
        self.result = DataResult::Payload(payload.into());
        self
    }

    /// # Errors
    /// Returns an error if the payload cannot be serialized.
    pub fn payload<S>(mut self, payload: S) -> Result<Self, rmpv::ext::Error>
    where
        S: Serialize,
    {
        let value = rmpv::ext::to_value(&payload)?;
        self.result = DataResult::Payload(value);
        Ok(self)
    }

    #[must_use]
    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = Some(channel);
        self
    }

    #[must_use]
    pub fn error(mut self, error: &impl ToString) -> Self {
        self.result = DataResult::Error(error.to_string());
        self
    }
}

pub enum RequestOrResponse {
    Request(RequestDataPack),
    Response(ResponseDataPack),
}

impl DataPack {
    /// Deserialize a `DataPack` from a byte slice.
    ///
    /// # Errors
    /// Returns an error if the byte slice is not a valid `DataPack`.
    pub fn deserialize(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(bytes)
    }

    /// Serialize a `DataPack` into a byte slice.
    ///
    /// # Errors
    /// Returns an error if the `DataPack` cannot be serialized.
    pub fn serialize(&self) -> Result<Bytes, rmp_serde::encode::Error> {
        rmp_serde::to_vec_named(self).map(Bytes::from)
    }

    /// Serialize a `DataPack` into a raw byte slice.
    ///
    /// # Errors
    /// Returns an error if the `DataPack` cannot be serialized.
    pub fn serialize_to_raw(&self) -> Result<RawDataPack, rmp_serde::encode::Error> {
        rmp_serde::to_vec_named(self).map(|v| RawDataPack::new(Bytes::from(v)))
    }

    #[must_use]
    pub const fn correlation(&self) -> Ulid {
        self.correlation
    }

    #[must_use]
    pub const fn is_request(&self) -> bool {
        matches!(self.header, Header::Request { .. })
    }

    #[must_use]
    pub const fn is_response(&self) -> bool {
        matches!(self.header, Header::Response)
    }

    #[must_use]
    pub fn into_req_or_rep(self) -> RequestOrResponse {
        let Self {
            correlation,
            header,
            channel,
            result,
        } = self;
        match header {
            Header::Request { path } => RequestOrResponse::Request(RequestDataPack {
                path,
                correlation,
                channel,
                payload: Result::from(result).unwrap_or_else(rmpv::Value::from),
            }),
            Header::Response => RequestOrResponse::Response(ResponseDataPack {
                correlation,
                channel,
                result,
            }),
        }
    }
}

pub struct DataPackCodec {
    raw: RawDataPackCodec,
}

impl DataPackCodec {
    #[must_use]
    pub fn new() -> Self {
        Self {
            raw: RawDataPackCodec::new(),
        }
    }
}

impl Default for DataPackCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for DataPackCodec {
    type Error = DataPackCodecError;
    type Item = DataPack;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let raw_data = self.raw.decode(src)?;
        let Some(raw_data) = raw_data else {
            return Ok(None);
        };
        Ok(Some(DataPack::deserialize(&raw_data.data)?))
    }
}

impl Encoder<&DataPack> for DataPackCodec {
    type Error = DataPackCodecError;

    fn encode(&mut self, item: &DataPack, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let raw_data = item.serialize_to_raw()?;
        self.raw.encode(raw_data, dst)?;
        Ok(())
    }
}

impl Encoder<RawDataPack> for DataPackCodec {
    type Error = DataPackCodecError;

    fn encode(&mut self, item: RawDataPack, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.raw.encode(item, dst)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum DataPackCodecError {
    #[error("IO error in DataPack codec: {0}")]
    IO(#[from] std::io::Error),
    #[error("DataPack serialization error: {0}")]
    Serialize(#[from] rmp_serde::encode::Error),
    #[error("DataPack deserialization error: {0}")]
    Deserialize(#[from] rmp_serde::decode::Error),
}
