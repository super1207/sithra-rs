use serde::{Deserialize, Serialize};
use sithra_server::{
    extract::context::{Clientful, Context},
    server::PostError,
};
use sithra_transport::datapack::RequestDataPack;
use smallvec::SmallVec;
use typeshare::typeshare;

#[typeshare]
#[derive(Deserialize, Serialize)]
pub struct Message {
    pub id:      String,
    #[typeshare(serialized_as = "Vec<Segment>")]
    pub content: SmallVec<[Segment; 1]>,
}

#[typeshare]
#[derive(Deserialize, Serialize)]
pub struct Segment {
    #[serde(rename = "type")]
    ty:      String,
    content: String,
}

#[typeshare]
#[derive(Deserialize, Serialize)]
pub struct SendMessage {
    #[typeshare(serialized_as = "Vec<Segment>")]
    pub content: SmallVec<[Segment; 1]>,
}

pub trait ContextExt {
    fn reply(
        &self,
        msg: SendMessage,
    ) -> impl Future<Output = Result<Message, PostError>> + Send + Sync;
}

impl<S> ContextExt for Context<Message, S>
where
    S: Clientful + Send + Sync,
{
    async fn reply(&self, msg: SendMessage) -> Result<Message, PostError> {
        let datapack = self
            .client()
            .post(
                RequestDataPack::default()
                    .path("/command/message.create")
                    .channel_opt(self.request.channel())
                    .payload(msg),
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
    typed!("/command/message.create" => impl SendMessage; SendMessage);
}
