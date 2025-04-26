use log::info;
use sithra_common::state::CommonState;

const SUBSCRIBERS: &[ioevent::Subscriber<CommonState>] = &[];

// TODO

#[sithra_common::main(subscribers = SUBSCRIBERS, state = CommonState)]
async fn main(_effect_wright: &ioevent::EffectWright) {
    info!("24 点游戏启动成功");
}
