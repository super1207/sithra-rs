export * as codec from "./codec";
export * as util from "./util";
export * as peer from "./peer";

export interface Channel {
  id: string,
  type: ChannelType,
  name: string,
  parent_id?: string,
}

export enum ChannelType {
  Group = "group",
  Direct = "direct",
  Private = "private",
}

export interface DataPack<T, R extends "response" | "request"> {
  path: R extends "response" ? never : string,
  correlation: string,
  channel?: Channel
  payload?: R extends "response" ? T | undefined : T,
  error?: R extends "response" ? string | undefined : never,
}

export type RequestDataPack<T> = DataPack<T, "request">;
export type ResponseDataPack<T> = DataPack<T, "response">;
