import type { DataPack } from "..";
export declare function encode(data: unknown): Uint8Array<ArrayBuffer>;
export declare function decodeFromRaw(buffer: Buffer): unknown;
export declare function tryDecodeFromRawWithLength(length: number, buffer: Buffer): [unknown, Buffer] | null;
export declare function tryDecodeFromRaw(buffer: Buffer): [unknown, Buffer] | null | number;
export interface Codec<T> {
    decode(chunk: Buffer): T | null;
    encode(data: T): Buffer;
}
export declare class DataPackCodec implements Codec<DataPack<unknown>> {
    deBuffer: Buffer;
    enBuffer: Buffer;
    dataLength: number | null;
    constructor();
    decode(chunk: Buffer): DataPack<unknown> | null;
    encode(data: DataPack<unknown>): Buffer;
}
//# sourceMappingURL=index.d.ts.map