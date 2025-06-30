use serde::{Deserialize, Serialize};
use sithra_server::{
    extract::context::{Clientful, Context},
    server::PostError,
};
use sithra_transport::{channel::Channel, datapack::RequestDataPack};
use smallvec::SmallVec;
use typeshare::typeshare;

#[typeshare]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub id:      String,
    #[typeshare(serialized_as = "Vec<SegmentType>")]
    pub content: SmallVec<[SegmentType; 1]>,
}

#[typeshare]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type", content = "parameter")]
pub enum SegmentType {
    /// Text segment, Content
    Text(String),
    /// Image segment, Source Url
    Image(String),
}

impl SegmentType {
    pub fn text<T: ToString>(content: &T) -> Self {
        Self::Text(content.to_string())
    }

    pub fn image<T: ToString>(url: &T) -> Self {
        Self::Image(url.to_string())
    }

    pub fn img<T: ToString>(url: &T) -> Self {
        Self::image(url)
    }
}

#[macro_export]
macro_rules! msg {
    [$($segment:ident: $value:expr),*$(,)?] => {
        [
            $(
                $crate::message::SegmentType::$segment($value),
            )*
        ].into_iter().collect::<$crate::smallvec::SmallVec<[$crate::message::SegmentType; 1]>>()
    };
}

#[typeshare]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendMessage {
    #[typeshare(serialized_as = "Vec<SegmentType>")]
    pub content: SmallVec<[SegmentType; 1]>,
}

impl SendMessage {
    #[must_use]
    pub const fn new(content: SmallVec<[SegmentType; 1]>) -> Self {
        Self { content }
    }
}

impl From<SmallVec<[SegmentType; 1]>> for SendMessage {
    fn from(content: SmallVec<[SegmentType; 1]>) -> Self {
        Self { content }
    }
}

pub trait ContextExt {
    fn reply(
        &self,
        msg: impl Into<SendMessage> + Send + Sync,
    ) -> impl Future<Output = Result<Message, PostError>> + Send + Sync;
}

impl<S> ContextExt for Context<Message, S>
where
    S: Clientful + Send + Sync,
{
    async fn reply(&self, msg: impl Into<SendMessage> + Send + Sync) -> Result<Message, PostError> {
        let datapack = self
            .client()
            .post(
                RequestDataPack::default()
                    .path("/command/message.create")
                    .channel_opt(self.request.channel())
                    .payload(msg.into()),
            )?
            .await?;
        let msg = datapack.payload::<Message>()?;
        Ok(msg)
    }
}

pub trait ClientfulExt {
    fn send_message(
        &self,
        channel: impl Into<Channel> + Send + Sync,
        msg: impl Into<SendMessage> + Send + Sync,
    ) -> impl Future<Output = Result<Message, PostError>> + Send + Sync;
}

impl<C> ClientfulExt for C
where
    C: Clientful + Send + Sync,
{
    async fn send_message(
        &self,
        channel: impl Into<Channel> + Send + Sync,
        msg: impl Into<SendMessage> + Send + Sync,
    ) -> Result<Message, PostError> {
        let datapack = self
            .client()
            .post(
                RequestDataPack::default()
                    .path("/command/message.create")
                    .channel(channel.into())
                    .payload(msg.into()),
            )?
            .await?;
        let msg = datapack.payload::<Message>()?;
        Ok(msg)
    }
}

pub mod event {
    use sithra_server::typed;

    use super::Message;
    typed!("/event/message.created" => impl Message; Message);
}

pub mod command {
    use sithra_server::typed;

    use super::SendMessage;
    use crate::into_response;
    typed!("/command/message.create" => impl SendMessage; SendMessage);

    into_response!("/command/message.create", SendMessage);
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use sithra_server::{
        extract::{
            context::{Clientful, Context as RawContext},
            payload::Payload,
            state::State,
        },
        handler::Handler,
        router,
        routing::router::Router,
        server::{Client, PostError},
    };
    use sithra_transport::channel::Channel;

    use super::Message;
    use crate::message::{ClientfulExt, ContextExt, SendMessage};

    #[derive(Clone)]
    struct AppState {
        client: Client,
    }

    type Context<T> = RawContext<T, AppState>;

    impl Clientful for AppState {
        fn client(&self) -> &Client {
            &self.client
        }
    }

    async fn on_message(ctx: Context<Message>) -> Result<(), PostError> {
        let _msg: &Message = ctx.payload();
        ctx.reply(msg![
            text: &"Hello, world!",
            img: &"https://example.com/image.png"
        ])
        .await?;
        Ok(())
    }

    async fn on_message2(channel: Channel, State(state): State<AppState>) -> Result<(), PostError> {
        state
            .send_message(
                channel,
                msg![
                    text: &"Hello, world!",
                    img: &"https://example.com/image.png"
                ],
            )
            .await?;
        Ok(())
    }

    async fn on_message3(Payload(_msg): Payload<Message>) -> SendMessage {
        msg![
            text: &"Hello, world!",
            img: &"https://example.com/image.png"
        ]
        .into()
    }

    #[tokio::test]
    async fn _type() {
        let _router = router! { Router::new() =>
            Message[on_message, on_message2, on_message3]
        };
    }
}
