export * as codec from "./codec";
export * as util from "./util";

export interface DataPack<T> {
  path: string,
  correlation: string,
  payload: T,
}
