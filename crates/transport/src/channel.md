# Channel

`Channel` 是一个通用的抽象，主要用于描述事件的来源。

## 数据格式

基本格式(JSON 示例)：

```jsonc
{
  "id": "用户的 ID", // 如果平台无法获取用户 ID，则使用事件 ID
  "name": "用户的名称", // 如果平台无法获取用户昵称，则使用 id 作为昵称
  "type": "group|direct|private",
  "parent_id": "父频道的 ID", // 可选
}
```

类型(TypeScript 示例):

```typescript
export interface Channel {
  id: string;
  type: ChannelType;
  name: string;
  parent_id?: string;
}

export enum ChannelType {
  Group = "group",
  Direct = "direct",
  Private = "private",
}
```

## Channel 类型

`type` 字段表示频道的类型，可以是 `group`、`direct` 或 `private`。以下案例可供参考(`parent_id` 未提及则可任意，适配器自行决定):

| 当事件来自     | `type`    | `id`              | `parent_id` |
| -------------- | --------- | ----------------- | ----------- |
| 私聊           | `private` | 用户 ID 或事件 ID |             |
| 组内用户       | `direct`  | 用户 ID 或事件 ID | 组 ID       |
| 组             | `group`   | 组 ID 或事件 ID   |             |
| 组的子组内用户 | `direct`  | 用户 ID 或事件 ID | 子组 ID     |
| 组的子组       | `group`   | 子组 ID 或事件 ID | 父组 ID     |

## 适用范围

- 事件: 当事件有明确的来源，则需包含 `channel` 字段标明来源。
- 行为: 当行为有明确的目的地，需要包含 `channel` 字段标明目的地。
- 信息: 当信息有明确的描述对象，需要包含 `channel` 字段标明描述了谁。

## kit 行为 & 适配器注意事项

当 Request 中包含 channel 字段时，路由返回的 Response 将会自动和 channel 关联。

例如: 当 `/message`

路由收到消息时，使用 kit 开发的插件可以:

- 直接返回 Request 作为回复/其他调用:
  此时 kit 将会自动将 Response 关联到 channel，以便适配器可以正确地处理消息。
- 通过 CALL 方法，手动关联 Request。

kit 和适配器于插件中是无状态的，意味着你不能简单的将整个 `bot` 实例作为参数传递给插件。

channel 只用于事件来源的关联。对于服务及其余需要内部状态关联的情况，参见 [correlation](crate::datapack::DataPack)。
