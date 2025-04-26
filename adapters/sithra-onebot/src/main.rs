use log::info;

const SUBSCRIBERS: &[ioevent::Subscriber<()>] = &[];

#[sithra_common::main(subscribers = SUBSCRIBERS, state = ())]
async fn main(_effect_wright: &ioevent::EffectWright) {
    info!("onebot 适配器启动成功");

    // 这样就可以获取到插件的数据路径
    let _data_path = sithra_common::data_path!();
}
