use log::info;
use sithra_common::message::common::CommonMessage;
use sithra_common::{msg, prelude::*};

const SUBSCRIBERS: &[ioevent::Subscriber<()>] = &[];

#[sithra_common::main(subscribers = SUBSCRIBERS, state = ())]
async fn main(wright: &ioevent::EffectWright) {
    info!("onebot 适配器启动成功");

    // 这样就可以获取到插件的数据路径
    let _data_path = sithra_common::data_path!();

    // 主循环
    loop {
        // 发送事件
        let channel = Channel::new(1234567890, ChannelType::Group);
        let user = User::empty();
        let message = msg!(CommonMessage[]);
        let event = MessageReceived::new(NonGenericId, channel, user, message);
        let result = wright.emit(&event);
        log::error!("something wrong: {:?}", result);
    }
}
