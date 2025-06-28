import { transport } from "sithra-types";
export * as codec from "./codec";
export * as util from "./util";
export * as peer from "./peer";

export interface IDataPack<T, R extends "response" | "request"> {
  path: R extends "response" ? never : string,
  correlation: string,
  channel?: transport.Channel,
  payload?: R extends "response" ? T | undefined : T,
  error?: R extends "response" ? string | undefined : never,
}

export type DataPack<T> = IDataPack<T, "response" | "request">;

export type RequestDataPack<T> = IDataPack<T, "request">;
