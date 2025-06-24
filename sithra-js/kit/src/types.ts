import { RequestDataPack, ResponseDataPack } from "sithra-transport";

export type Handler<T, R> = (request: RequestDataPack<T>) => R extends void ? void : ResponseDataPack<R>;

export type RouteType = {
  "/message": Handler<string, void>
  "/other": Handler<string, string>
}

export type HandlerRequest<H> = H extends Handler<infer T, infer _> ? RequestDataPack<T> : never;

export type HandlerResponse<H> = H extends Handler<infer _, infer R> ? (R extends void ? void : ResponseDataPack<R>) : never;

export type HandlerFunction<H> = H extends Handler<infer T, infer R> ? (request: RequestDataPack<T>) => R extends void ? void : ResponseDataPack<R> : never;

// export type AssertEqual<A, B> = A extends B ? B extends A ? true : never : never;

// type Test = AssertEqual<Handler<string, void>, HandlerFunction<Handler<string, void>>>;
