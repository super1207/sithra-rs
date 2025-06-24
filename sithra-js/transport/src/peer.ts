import { DataPack, RequestDataPack, ResponseDataPack } from ".";
import { IDataPackCodec } from "./codec";
import { asChunks, initStdio } from "./util";

export class Peer {
  codec: IDataPackCodec
  buffer: Buffer
  listeners: Array<(data: RequestDataPack<unknown>) => void>
  constructor(codec: IDataPackCodec) {
    this.codec = codec;
    this.buffer = Buffer.alloc(0);
    this.listeners = [];
    initStdio()
    process.stdin.on("data", (
      data: Buffer
    ) => {
      let decoded = codec.decode(data)
      if (decoded) {
        this.listeners.forEach(listener => listener(decoded));
      }
    });
  }
  onData(callback: (data: RequestDataPack<unknown>) => void) {
    this.listeners.push(callback);
  }
  route(path: string, callback: (data: RequestDataPack<unknown>) => void) {
    this.onData((data) => {
      if (data.path === path) {
        callback(data)
      }
    })
  }
  async send(data: ResponseDataPack<unknown>) {
    const buffer = this.codec.encode(data);
    const buffers = asChunks(buffer);
    for (const chunk of buffers) {
      process.stdout.write(chunk);
    }
  }
}
