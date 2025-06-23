import type { DataPack, RequestDataPack, ResponseDataPack } from ".."
import { encode as msgpackEncode, decode as msgpackDecode } from "@msgpack/msgpack";

export function encode(data: unknown): Uint8Array<ArrayBuffer> {
  let dataRaw = msgpackEncode(data);
  const totalLength = 4 + dataRaw.length;
  const buffer = new ArrayBuffer(totalLength);
  const view = new DataView(buffer);
  view.setUint32(0, dataRaw.length, false);
  const result = new Uint8Array(buffer);
  result.set(dataRaw, 4);

  return result;
}

export function decodeFromRaw(buffer: Buffer): unknown {
  return msgpackDecode(buffer);
}

export function tryDecodeFromRawWithLength(length: number, buffer: Buffer): [unknown, Buffer] | null {
  if (buffer.byteLength < length) {
    return null;
  }
  const data = buffer.subarray(0, length);
  return [decodeFromRaw(data), buffer.subarray(length)];
}

export function tryDecodeFromRaw(buffer: Buffer): [unknown, Buffer] | null | number {
  if (buffer.byteLength < 4) {
    return null;
  }
  const length = buffer.readUInt32BE(0);
  return tryDecodeFromRawWithLength(length, buffer) ?? length;
}

export interface Codec<D, E> {
  decode(chunk: Buffer): D | null;
  encode(data: E): Buffer;
}

export class DataPackCodec implements Codec<RequestDataPack<unknown>, ResponseDataPack<unknown>> {
  deBuffer: Buffer
  enBuffer: Buffer
  dataLength: number | null

  constructor() {
    this.deBuffer = Buffer.from([]);
    this.enBuffer = Buffer.from([]);
    this.dataLength = null;
  }

  decode(chunk: Buffer): RequestDataPack<unknown> | null {
    this.deBuffer = Buffer.concat([this.deBuffer, chunk]);
    if (this.deBuffer.byteLength <= 0) {
      return null;
    }
    if (this.dataLength == null) {
      if (this.deBuffer.byteLength < 4) {
        return null;
      }
      this.dataLength = this.deBuffer.readUInt32BE(0);
      this.deBuffer = this.deBuffer.subarray(4);
    }
    let [data, remainingBuffer] = tryDecodeFromRawWithLength(this.dataLength, this.deBuffer) ?? [null, this.deBuffer];
    this.deBuffer = remainingBuffer;
    this.dataLength = null;
    if (!(data as any)["path"]) {
      return null;
    }
    return data as RequestDataPack<unknown>;
  }

  encode(data: ResponseDataPack<unknown>): Buffer {
    return Buffer.from(encode(data))
  }
}
