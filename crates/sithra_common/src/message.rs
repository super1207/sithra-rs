use micromap::Map;
use serde::{Deserialize, Deserializer, Serialize};

use crate::model::{KV, MessageId, SVec};

/// 原始消息段，其中 kv 仅可最多包含 12 个字符串键值对，用于存储消息内容
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SegmentRaw {
    pub r#type: String,
    pub kv: Map<String, String, 3>,
}

impl SegmentRaw {
    pub fn new(r#type: impl ToString, kv: KV) -> Self {
        Self { r#type: r#type.to_string(), kv }
    }
    // 基础消息段类型构造器
    /// 文本
    pub fn text(text: impl ToString) -> Self {
        let mut kv = KV::new();
        kv.insert("content".to_string(), text.to_string());
        Self::new("text".to_string(), kv)
    }
    /// 图片
    pub fn img(url: impl ToString) -> Self {
        let mut kv = KV::new();
        kv.insert("url".to_string(), url.to_string());
        Self::new("image".to_string(), kv)
    }
    /// 提及用户
    pub fn at(user_id: impl ToString) -> Self {
        let mut kv = KV::new();
        kv.insert("user_id".to_string(), user_id.to_string());
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
    Self: IntoIterator<Item = Self::Segment> + Clone + for<'a> Deserialize<'a> + Serialize,
{
    /// 消息段类型
    type Segment: Segment;
    /// 从原始消息段列表中生成消息段迭代器
    fn segments(raw: impl IntoIterator<Item = SegmentRaw>) -> impl Iterator<Item = Self::Segment> {
        raw.into_iter()
            .filter_map(<Self::Segment as Segment>::Deserializer::deserialize)
    }
    /// 从原始消息段列表和消息 ID 生成消息
    fn from_raw(raw: MessageRaw) -> Self;
    /// 将消息转换为原始消息段列表和消息 ID
    fn into_raw(self) -> MessageRaw {
        let id = self.id();
        let segments = self
            .into_iter()
            .filter_map(<Self::Segment as Segment>::Serializer::serialize)
            .collect::<SVec<_>>();
        MessageRaw::new(segments, id)
    }
    /// 获取消息 ID
    fn id(&self) -> Option<MessageId>;
    /// 从消息段数组生成消息，仅用于发送！
    fn from_array<const N: usize>(array: [Self::Segment; N]) -> Self;
}
/// 原始消息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageRaw {
    pub segments: SVec<SegmentRaw>,
    pub id: Option<MessageId>,
}
impl MessageRaw {
    /// 从消息段数组和消息 ID 生成原始消息
    pub fn new(segments: SVec<SegmentRaw>, id: Option<MessageId>) -> Self {
        Self { segments, id }
    }
}
impl From<MessageRaw> for SVec<SegmentRaw> {
    fn from(value: MessageRaw) -> Self {
        value.segments
    }
}
impl<M: Message> From<M> for MessageRaw {
    fn from(value: M) -> Self {
        let id = value.id();
        let segments = value.into_raw().into();
        Self::new(segments, id)
    }
}
pub trait FromRawSegment
where
    Self: Sized + Segment,
{
    fn from_raw_segment(segment: &mut SegmentRaw) -> Option<Self>;
}
impl<T: FromRawSegment + Segment> MessageDeserializer for T {
    type Output = Self;
    fn deserialize(mut segment: SegmentRaw) -> Option<Self> {
        let kind = Self::from_raw_segment(&mut segment)?;
        Some(kind)
    }
}

pub mod common {
    use crate::model::UserId;

    use super::*;
    /// 一般消息段类型
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum CommonSegment {
        /// 文本(文本内容)
        Text(String),
        /// 图片(图片 URL)
        Image(String),
        /// 提及用户(用户 ID)
        At(UserId),
        /// 未知消息段
        Unknown(SegmentRaw),
    }
    impl CommonSegment {
        /// 生成文本消息段
        pub fn text<S: ToString>(text: S) -> Self {
            Self::Text(text.to_string())
        }
        /// 生成图片消息段
        pub fn img<S: ToString>(url: S) -> Self {
            Self::Image(url.to_string())
        }
        /// 生成提及用户消息段
        pub fn at<S: Into<UserId>>(user_id: S) -> Self {
            Self::At(user_id.into())
        }
        pub fn unknown<S: ToString>(r#type: S, kv: KV) -> Self {
            Self::Unknown(SegmentRaw::new(r#type.to_string(), kv))
        }
    }
    impl FromRawSegment for CommonSegment {
        fn from_raw_segment(segment: &mut SegmentRaw) -> Option<Self> {
            match segment.r#type.as_str() {
                "text" => Some(CommonSegment::Text(segment.kv.remove("content")?)),
                "image" => Some(CommonSegment::Image(segment.kv.remove("url")?)),
                "at" => Some(CommonSegment::At(UserId::new(
                    segment.kv.remove("user_id")?,
                ))),
                _ => None,
            }
        }
    }
    impl Segment for CommonSegment {
        type Serializer = CommonMessageSerializer;
        type Deserializer = CommonSegment;
    }
    /// 一般消息类型。
    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct CommonMessage {
        /// 消息 ID
        id: Option<MessageId>,
        /// 消息段
        inner: SVec<CommonSegment>,
    }
    pub struct CommonMessageSerializer;
    impl MessageSerializer for CommonMessageSerializer {
        type Input = CommonSegment;
        fn serialize(message: Self::Input) -> Option<SegmentRaw> {
            match message {
                CommonSegment::Text(text) => Some(SegmentRaw::text(text)),
                CommonSegment::Image(url) => Some(SegmentRaw::img(url)),
                CommonSegment::At(user_id) => Some(SegmentRaw::at(user_id.to_string())),
                CommonSegment::Unknown(segment) => Some(segment),
            }
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
        fn id(&self) -> Option<MessageId> {
            self.id.clone()
        }
        fn from_raw(raw: MessageRaw) -> Self {
            let segments = Self::segments(raw.segments).collect();
            Self {
                id: raw.id,
                inner: segments,
            }
        }
        fn from_array<const N: usize>(array: [Self::Segment; N]) -> Self {
            let segments = array.into_iter().collect();
            Self {
                id: None,
                inner: segments,
            }
        }
    }
}
/// 反序列化消息
pub fn deserialize_message<'de, D, M>(deserializer: D) -> Result<M, D::Error>
where
    D: Deserializer<'de>,
    M: Message,
{
    let raw = MessageRaw::deserialize(deserializer)?;
    Ok(M::from_raw(raw))
}
/// 接收消息段类型和消息段列表，返回消息类型
/// 例子：
/// ```rust
/// let msg = msg!(CommonMessage[
///     text: "Hello, world!",
///     img: "https://example.com/image.png",
///     at: "1234567890",
/// ]);
/// ```
#[macro_export]
macro_rules! msg {
    ($type:ident[$($segment:ident: $value:expr),*$(,)?]) => {
        $type::from_array([
            $(
                <$type as $crate::message::Message>::Segment::$segment($value),
            )*
        ])
    };
}

pub fn create_kv<M: ToString, const N: usize>(value: [(M, M); N]) -> KV {
    KV::from_iter(
        value
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string())),
    )
}

/// 将键值对转换为KV
///
/// # 示例
///
/// ```rust
/// let map = kv!{
///     "key1": "value1",
///     "key2": "value2",
/// };
/// ```
#[macro_export]
macro_rules! kv {
    {$($key:tt : $value:expr),* $(,)?} => {
        $crate::message::create_kv([
            $(($key, $value)),*
        ])
    };
}
