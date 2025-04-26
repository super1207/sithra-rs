use micromap::Map;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

pub type KV = Map<String, String, 3>;
pub type SVec<T> = SmallVec<[T; 3]>;

/// 原始消息段，其中 kv 仅可最多包含 12 个键值对，用于存储消息内容
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SegmentRaw {
    pub r#type: String,
    pub message_id: String,
    pub kv: Map<String, String, 3>,
}

impl SegmentRaw {
    pub fn new(r#type: String, message_id: String, kv: KV) -> Self {
        Self {
            r#type,
            message_id,
            kv,
        }
    }
    // 基础消息段类型构造器
    /// 文本
    pub fn text(message_id: String, text: String) -> Self {
        let mut kv = KV::new();
        kv.insert("content".to_string(), text);
        Self::new("text".to_string(), message_id, kv)
    }
    /// 图片
    pub fn image(message_id: String, url: String) -> Self {
        let mut kv = KV::new();
        kv.insert("url".to_string(), url);
        Self::new("image".to_string(), message_id, kv)
    }
    /// 提及用户
    pub fn at(message_id: String, user_id: String) -> Self {
        let mut kv = KV::new();
        kv.insert("user_id".to_string(), user_id);
        Self::new("at".to_string(), message_id, kv)
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
    Self: IntoIterator<Item = Self::Segment> + FromIterator<Self::Segment> + Clone,
{
    type Segment: Segment;
    fn from_raw_iter(raw: impl IntoIterator<Item = SegmentRaw>) -> Self {
        raw.into_iter()
            .filter_map(<Self::Segment as Segment>::Deserializer::deserialize)
            .collect()
    }
    fn from_raw(raw: MessageRaw) -> Self {
        Self::from_raw_iter(raw.segments)
    }
    fn into_raw(self) -> SVec<SegmentRaw> {
        self.into_iter()
            .filter_map(<Self::Segment as Segment>::Serializer::serialize)
            .collect()
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageRaw {
    pub segments: SVec<SegmentRaw>,
}
impl MessageRaw {
    pub fn new(segments: SVec<SegmentRaw>) -> Self {
        Self { segments }
    }
}
impl From<MessageRaw> for SVec<SegmentRaw> {
    fn from(value: MessageRaw) -> Self {
        value.segments
    }
}
impl<T: Message> From<T> for MessageRaw {
    fn from(value: T) -> Self {
        Self::new(value.into_raw())
    }
}

pub mod common {
    use super::*;
    #[derive(Debug, Clone)]
    pub struct CommonSegment {
        pub id: String,
        pub kind: CommonSegmentKind,
    }
    #[derive(Debug, Clone)]
    pub enum CommonSegmentKind {
        /// 文本
        Text(String),
        /// 图片
        Image(String),
        /// 提及用户
        At(String),
    }
    impl From<&mut SegmentRaw> for Option<CommonSegmentKind> {
        fn from(value: &mut SegmentRaw) -> Self {
            match value.r#type.as_str() {
                "text" => Some(CommonSegmentKind::Text(value.kv.remove("content")?)),
                "image" => Some(CommonSegmentKind::Image(value.kv.remove("url")?)),
                "at" => Some(CommonSegmentKind::At(value.kv.remove("user_id")?)),
                _ => None,
            }
        }
    }
    pub struct CommonMessageProcessor;
    impl MessageSerializer for CommonMessageProcessor {
        type Input = CommonSegment;
        fn serialize(message: Self::Input) -> Option<SegmentRaw> {
            match message.kind {
                CommonSegmentKind::Text(text) => Some(SegmentRaw::text(message.id, text)),
                CommonSegmentKind::Image(url) => Some(SegmentRaw::image(message.id, url)),
                CommonSegmentKind::At(user_id) => Some(SegmentRaw::at(message.id, user_id)),
            }
        }
    }
    impl MessageDeserializer for CommonMessageProcessor {
        type Output = CommonSegment;
        fn deserialize(mut segment: SegmentRaw) -> Option<Self::Output> {
            let kind = Option::<CommonSegmentKind>::from(&mut segment)?;
            Some(CommonSegment {
                id: segment.message_id,
                kind,
            })
        }
    }
    impl Segment for CommonSegment {
        type Serializer = CommonMessageProcessor;
        type Deserializer = CommonMessageProcessor;
    }
    /// 一般消息类型。
    #[derive(Debug, Clone)]
    pub struct CommonMessage {
        inner: SVec<CommonSegment>,
    }
    impl IntoIterator for CommonMessage {
        type Item = CommonSegment;
        type IntoIter = smallvec::IntoIter<[Self::Item; 3]>;
        fn into_iter(self) -> Self::IntoIter {
            self.inner.into_iter()
        }
    }
    impl FromIterator<CommonSegment> for CommonMessage {
        fn from_iter<T: IntoIterator<Item = CommonSegment>>(iter: T) -> Self {
            Self {
                inner: iter.into_iter().collect(),
            }
        }
    }
    impl Message for CommonMessage {
        type Segment = CommonSegment;
    }
}
