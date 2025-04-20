# **`sithra-rs`**

**`sithra-rs`** 是一个基于 Rust 实现的 OneBot 聊天机器人框架，专注于提供高性能和可扩展的机器人开发体验。

## 项目结构

项目采用 Cargo workspace 组织，包含以下主要组件：

- `crates/sithra`: 核心框架实现
- `crates/sithra_common`: 公共类型和工具
- `crates/sithra_macro`: 过程宏支持
- `examples/`: 示例插件

## 特性

- 完整的 OneBot 协议支持
- 基于 Tokio 的异步运行时
- 事件驱动架构
- 插件系统支持
- 类型安全的设计

## 快速开始

[待补充]

## 插件开发

开始开发 `sithra-rs` 的插件，需要为你的 crate 添加这些依赖：

```toml
ioevent = { git = "https://github.com/BERADQ/ioevent.git" }
sithra_common = { git = "https://github.com/SithraBot/sithra-rs.git" }
tokio = "*"
log = "*"
```

一个简单的 echo 插件精神面貌是这样的：

```rust
use event::MessageEventFlattened as Message;
use ioevent::{prelude::*, rpc::*};
use log::info;
use sithra_common::prelude::*;

const SUBSCRIBERS: &[ioevent::Subscriber<CommonState>] = &[
    create_subscriber!(echo_msg)
];

#[subscribe_message]
async fn echo_msg(msg: &Message) -> Option<Vec<MessageNode>> {
    if msg.starts_with("echo ") {
        info!("echo 插件收到{}发送的消息", msg.sender.call_name());
        let message = msg.message.clone().trim_start_matches("echo ");
        return Some(message);
    }
    None
}

#[sithra_common::main(subscribers = SUBSCRIBERS, state = CommonState)]
async fn main(_effect_wright: &ioevent::EffectWright) {
    info!("echo 示例插件启动成功");
}
```

## 文档

`sithra-rs` 的一切都离不开 [`ioevent`](https://github.com/BERADQ/ioevent)。

对于插件开发的一些基本知识，请参阅 [`ioevent` doc](https://docs.rs/ioevent/latest/ioevent/) **(过时)**。

[待补充]

## 许可证

[Unlicense](https://github.com/SithraBot/sithra-rs/blob/main/LICENSE)

## 贡献

欢迎提交 Issue 和 Pull Request！

## 社区

- [QQ 群](https://qm.qq.com/q/XtORRK5Ruk)

## 发音

`/siːθrə/`