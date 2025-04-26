use micromap::Map;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

pub type KV = Map<String, String, 3>;
pub type SVec<T> = SmallVec<[T; 3]>;

/// 原始消息段，其中 kv 仅可最多包含 12 个键值对，用于存储消息内容
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SegmentRaw {
    pub r#type: String,
    pub kv: Map<String, String, 3>,
}

impl SegmentRaw {
    pub fn new(r#type: String, kv: KV) -> Self {
        Self { r#type, kv }
    }
    // 基础消息段类型构造器
    /// 文本
    pub fn text(text: String) -> Self {
        let mut kv = KV::new();
        kv.insert("content".to_string(), text);
        Self::new("text".to_string(), kv)
    }
    /// 图片
    pub fn image(url: String) -> Self {
        let mut kv = KV::new();
        kv.insert("url".to_string(), url);
        Self::new("image".to_string(), kv)
    }
    /// 提及用户
    pub fn at(user_id: String) -> Self {
        let mut kv = KV::new();
        kv.insert("user_id".to_string(), user_id);
        Self::new("at".to_string(), kv)
    }
}
/// 消息段类型
pub trait Segment
where
    Self: Sized + Clone + Send,
{
    type Serializer: MessageSerializer<Input = Self>;
    type Deserializer: MessageDeserializer<Output = Self>;
}
/// 消息段反序列化器
pub trait MessageDeserializer {
    type Output: Segment;
    fn deserialize(segment: SegmentRaw) -> Option<Self::Output>;
}
/// 消息序列化器
pub trait MessageSerializer {
    type Input: Segment;
    fn serialize(message: Self::Input) -> Option<SegmentRaw>;
}
/// 消息类型
pub trait Message
where
    Self: IntoIterator<Item = Self::Segment> + Clone,
{
    type Segment: Segment;
    fn segments(
        raw: impl IntoIterator<Item = SegmentRaw>,
    ) -> impl Iterator<Item = Self::Segment> {
        raw.into_iter()
            .filter_map(<Self::Segment as Segment>::Deserializer::deserialize)
    }
    fn from_raw(raw: MessageRaw) -> Self;
    fn into_raw(self) -> MessageRaw {
        let id = self.id();
        let segments = self
            .into_iter()
            .filter_map(<Self::Segment as Segment>::Serializer::serialize)
            .collect::<SVec<_>>();
        MessageRaw::new(segments, id)
    }
    fn id(&self) -> Option<String>;
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageRaw {
    pub segments: SVec<SegmentRaw>,
    pub id: Option<String>,
}
impl MessageRaw {
    pub fn new(segments: SVec<SegmentRaw>, id: Option<String>) -> Self {
        Self { segments, id }
    }
}
impl From<MessageRaw> for SVec<SegmentRaw> {
    fn from(value: MessageRaw) -> Self {
        value.segments
    }
}
impl<T: Message> From<T> for MessageRaw {
    fn from(value: T) -> Self {
        let id = value.id();
        let segments = value.into_raw().into();
        Self::new(segments, id)
    }
}

pub mod common {
    use super::*;
    #[derive(Debug, Clone)]
    pub enum CommonSegment {
        /// 文本
        Text(String),
        /// 图片
        Image(String),
        /// 提及用户
        At(String),
    }
    impl From<&mut SegmentRaw> for Option<CommonSegment> {
        fn from(value: &mut SegmentRaw) -> Self {
            match value.r#type.as_str() {
                "text" => Some(CommonSegment::Text(value.kv.remove("content")?)),
                "image" => Some(CommonSegment::Image(value.kv.remove("url")?)),
                "at" => Some(CommonSegment::At(value.kv.remove("user_id")?)),
                _ => None,
            }
        }
    }
    impl Segment for CommonSegment {
        type Serializer = CommonMessageProcessor;
        type Deserializer = CommonMessageProcessor;
    }
    /// 一般消息类型。
    #[derive(Debug, Clone)]
    pub struct CommonMessage {
        pub id: Option<String>,
        inner: SVec<CommonSegment>,
    }
    pub struct CommonMessageProcessor;
    impl MessageSerializer for CommonMessageProcessor {
        type Input = CommonSegment;
        fn serialize(message: Self::Input) -> Option<SegmentRaw> {
            match message {
                CommonSegment::Text(text) => Some(SegmentRaw::text(text)),
                CommonSegment::Image(url) => Some(SegmentRaw::image(url)),
                CommonSegment::At(user_id) => Some(SegmentRaw::at(user_id)),
            }
        }
    }
    impl MessageDeserializer for CommonMessageProcessor {
        type Output = CommonSegment;
        fn deserialize(mut segment: SegmentRaw) -> Option<Self::Output> {
            let kind = Option::<CommonSegment>::from(&mut segment)?;
            Some(kind)
        }
    }
    impl IntoIterator for CommonMessage {
        type Item = CommonSegment;
        type IntoIter = smallvec::IntoIter<[Self::Item; 3]>;
        fn into_iter(self) -> Self::IntoIter {
            self.inner.into_iter()
        }
    }
    impl Message for CommonMessage {
        type Segment = CommonSegment;
        fn id(&self) -> Option<String> {
            self.id.clone()
        }
        fn from_raw(raw: MessageRaw) -> Self {
            let segments = Self::segments(raw.segments).collect();
            Self {
                id: raw.id,
                inner: segments,
            }
        }
    }
}
