use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Channel {
    pub id:        String,
    #[serde(rename = "type")]
    pub ty:        ChannelType,
    pub name:      String,
    pub parent_id: Option<String>,
}

impl Channel {
    #[allow(non_snake_case)]
    #[must_use]
    pub const fn Private(id: String, name: String) -> Self {
        Self {
            id,
            ty: ChannelType::Private,
            name,
            parent_id: None,
        }
    }

    #[allow(non_snake_case)]
    #[must_use]
    pub const fn Group(id: String, name: String) -> Self {
        Self {
            id,
            ty: ChannelType::Group,
            name,
            parent_id: None,
        }
    }

    #[allow(non_snake_case)]
    #[must_use]
    pub const fn Direct(id: String, name: String) -> Self {
        Self {
            id,
            ty: ChannelType::Direct,
            name,
            parent_id: None,
        }
    }

    #[allow(non_snake_case)]
    #[must_use]
    pub const fn DirectFromGroup(group_id: String, id: String, name: String) -> Self {
        Self {
            id,
            ty: ChannelType::Direct,
            name,
            parent_id: Some(group_id),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    Group,
    Direct,
    Private,
}
