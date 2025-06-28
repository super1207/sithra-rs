import { parse } from "regexparam"
import { RequestDataPack } from "sithra-transport";
import { HandlerFunction, HandlerRequest, HandlerResponse, RouteType } from "./types";

export type Route = {
  keys: string[];
  pattern: RegExp;
};


export class Router {
  routes: (Route & { symbol: symbol })[];
  map: Map<Symbol, RouteType[keyof RouteType]>

  constructor() {
    this.routes = [];
    this.map = new Map();
  }

  #add<P extends keyof RouteType>(route: P, value: RouteType[P]): void {
    const route_ = parse(route);
    const symbol = Symbol();
    this.routes.push({ ...route_, symbol });
    this.map.set(symbol, value);
  }

  #exec(path: string, route: Route): Record<string, string> {
    let i = 0, out: Record<string, string> = {};
    let matches = route.pattern.exec(path);
    while (i < route.keys.length && matches) {
      out[route.keys[i]] = matches[++i];
    }
    return out;
  }

  #getExec(path: string): {
    value: RouteType[keyof RouteType];
    params: Record<string, string>;
  } | undefined {
    const route = this.routes.find(({ pattern }) => pattern.test(path));
    return route ? { value: this.map.get(route.symbol)!!, params: this.#exec(path, route) } : undefined;
  }

  #get<P extends keyof RouteType>(path: P): RouteType[P] | undefined {
    const route = this.routes.find(({ pattern }) => pattern.test(path));
    return route ? this.map.get(route.symbol) as RouteType[P] : undefined;
  }

  route<P extends keyof RouteType>(path: P, handler: RouteType[P]): void {
    this.#add(path, handler);
  }

  call<P extends keyof RouteType>(path: P, request: HandlerRequest<RouteType[P]>): HandlerResponse<RouteType[P]> | undefined {
    const endpoint = this.#get(path);
    return endpoint?.(request) as HandlerResponse<RouteType[P]>;
  }
}
