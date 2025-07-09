use de::Error as _;
use ser::Error as _;
use serde::{Deserialize, Serialize, de, ser};
use sithra_kit::types::message::{NIL, Segment};

use crate::message::internal::{
    Contact, InternalOneBotSegment, InternalOneBotUnknown, Location, Poke,
};

pub mod internal {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(rename_all = "snake_case", tag = "type", content = "data")]
    pub enum InternalOneBotSegment {
        Text {
            text: String,
        },
        Face {
            id: String,
        },
        Image {
            file: String,
        },
        Record {
            file: String,
        },
        Video {
            file: String,
        },
        At {
            #[serde(default)]
            id: String,
            #[serde(default)]
            qq: String,
        },
        Rps,
        Dice,
        Shake,
        Poke(Poke),
        Contact(Contact),
        Location(Location),
        Reply {
            id: String,
        },
        #[serde(other)]
        Unknown,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Poke {
        #[serde(rename = "type")]
        pub ty: String,
        pub id: String,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Contact {
        #[serde(rename = "type")]
        pub ty: String,
        pub id: String,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Location {
        pub lat: f64,
        pub lon: f64,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct InternalOneBotUnknown {
        #[serde(rename = "type")]
        pub ty:   String,
        pub data: rmpv::Value,
    }
}

#[derive(Debug, Clone)]
pub enum OneBotSegment {
    Typed(InternalOneBotSegment),
    Unknown(InternalOneBotUnknown),
}

impl OneBotSegment {
    pub fn text<T: ToString>(content: &T) -> Self {
        Self::Typed(InternalOneBotSegment::Text {
            text: content.to_string(),
        })
    }

    pub fn image<T: ToString>(url: &T) -> Self {
        Self::Typed(InternalOneBotSegment::Image {
            file: url.to_string(),
        })
    }

    pub fn img<T: ToString>(url: &T) -> Self {
        Self::image(url)
    }

    pub fn at<T: ToString>(target: &T) -> Self {
        Self::Typed(InternalOneBotSegment::At {
            id: target.to_string(),
            qq: target.to_string(),
        })
    }

    pub fn reply<T: ToString>(target: &T) -> Self {
        Self::Typed(InternalOneBotSegment::Reply {
            id: target.to_string(),
        })
    }

    #[must_use]
    pub const fn location((lat, lon): (f64, f64)) -> Self {
        Self::Typed(InternalOneBotSegment::Location(Location { lat, lon }))
    }

    pub fn face<T: ToString>(id: &T) -> Self {
        Self::Typed(InternalOneBotSegment::Face { id: id.to_string() })
    }

    pub fn video<T: ToString>(url: &T) -> Self {
        Self::Typed(InternalOneBotSegment::Video {
            file: url.to_string(),
        })
    }

    pub fn record<T: ToString>(url: &T) -> Self {
        Self::Typed(InternalOneBotSegment::Record {
            file: url.to_string(),
        })
    }

    #[must_use]
    pub const fn rps() -> Self {
        Self::Typed(InternalOneBotSegment::Rps)
    }

    #[must_use]
    pub const fn dice() -> Self {
        Self::Typed(InternalOneBotSegment::Dice)
    }

    #[must_use]
    pub const fn shake() -> Self {
        Self::Typed(InternalOneBotSegment::Shake)
    }

    pub fn poke<T1: ToString, T2: ToString>((ty, id): (&T1, &T2)) -> Self {
        Self::Typed(InternalOneBotSegment::Poke(Poke {
            ty: ty.to_string(),
            id: id.to_string(),
        }))
    }

    pub fn contact<T1: ToString, T2: ToString>((ty, id): (&T1, &T2)) -> Self {
        Self::Typed(InternalOneBotSegment::Contact(Contact {
            ty: ty.to_string(),
            id: id.to_string(),
        }))
    }
}

impl TryFrom<Segment> for OneBotSegment {
    type Error = rmpv::ext::Error;

    fn try_from(value: Segment) -> Result<Self, Self::Error> {
        let Segment { ty, data } = value;
        match ty.as_str() {
            "text" => Ok(Self::Typed(InternalOneBotSegment::Text {
                text: rmpv::ext::from_value(data)?,
            })),
            "face" => Ok(Self::Typed(InternalOneBotSegment::Face {
                id: rmpv::ext::from_value(data)?,
            })),
            "image" => Ok(Self::Typed(InternalOneBotSegment::Image {
                file: rmpv::ext::from_value(data)?,
            })),
            "record" => Ok(Self::Typed(InternalOneBotSegment::Record {
                file: rmpv::ext::from_value(data)?,
            })),
            "video" => Ok(Self::Typed(InternalOneBotSegment::Video {
                file: rmpv::ext::from_value(data)?,
            })),
            "at" => {
                let id: String = rmpv::ext::from_value(data)?;
                Ok(Self::Typed(InternalOneBotSegment::At {
                    id: id.clone(),
                    qq: id,
                }))
            }
            "rps" => Ok(Self::Typed(InternalOneBotSegment::Rps)),
            "dice" => Ok(Self::Typed(InternalOneBotSegment::Dice)),
            "shake" => Ok(Self::Typed(InternalOneBotSegment::Shake)),
            "poke" => Ok(Self::Typed(InternalOneBotSegment::Poke(
                rmpv::ext::from_value(data)?,
            ))),
            "contact" => Ok(Self::Typed(InternalOneBotSegment::Contact(
                rmpv::ext::from_value(data)?,
            ))),
            "location" => Ok(Self::Typed(InternalOneBotSegment::Location(
                rmpv::ext::from_value(data)?,
            ))),
            "reply" => Ok(Self::Typed(InternalOneBotSegment::Reply {
                id: rmpv::ext::from_value(data)?,
            })),
            "unknown" => Ok(Self::Typed(InternalOneBotSegment::Unknown)),
            _ => Ok(Self::Unknown(InternalOneBotUnknown { ty, data })),
        }
    }
}

impl TryFrom<OneBotSegment> for Segment {
    type Error = rmpv::ext::Error;

    fn try_from(value: OneBotSegment) -> Result<Self, Self::Error> {
        match value {
            OneBotSegment::Typed(typed) => match typed {
                InternalOneBotSegment::Text { text } => Ok(Self::text(&text)),
                InternalOneBotSegment::Face { id } => Self::custom(&"face", id),
                InternalOneBotSegment::Image { file } => Ok(Self::image(&file)),
                InternalOneBotSegment::Record { file } => Self::custom(&"file", file),
                InternalOneBotSegment::Video { file } => Self::custom(&"video", file),
                InternalOneBotSegment::At { id, qq } => {
                    if qq.is_empty() {
                        Ok(Self::at(&id))
                    } else {
                        Ok(Self::at(&qq))
                    }
                }
                InternalOneBotSegment::Rps => Self::custom(&"rps", NIL),
                InternalOneBotSegment::Dice => Self::custom(&"dice", NIL),
                InternalOneBotSegment::Shake => Self::custom(&"shake", NIL),
                InternalOneBotSegment::Reply { id } => Self::custom(&"reply", id),
                InternalOneBotSegment::Poke(poke) => Self::custom(&"poke", poke),
                InternalOneBotSegment::Contact(contact) => Self::custom(&"contact", contact),
                InternalOneBotSegment::Location(location) => Self::custom(&"location", location),
                InternalOneBotSegment::Unknown => Self::custom(&"unknown", NIL),
            },
            OneBotSegment::Unknown(unknown) => Ok(Self {
                ty:   unknown.ty,
                data: unknown.data,
            }),
        }
    }
}

impl<'de> Deserialize<'de> for OneBotSegment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
        D::Error: de::Error,
    {
        let raw = Segment::deserialize(deserializer)?;
        raw.try_into().map_err(|e| D::Error::custom(format!("Invalid segment: {e}")))
    }
}

impl Serialize for OneBotSegment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        S::Error: ser::Error,
    {
        let segment: Segment = self
            .clone()
            .try_into()
            .map_err(|e| S::Error::custom(format!("Failed to serialize segment: {e}")))?;
        segment.serialize(serializer)
    }
}
