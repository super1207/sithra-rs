import { DataPack } from ".";
import { Codec } from "./codec";
import { asChunks, initStdio } from "./util";

export class Peer {
  codec: Codec<DataPack<unknown>>
  buffer: Buffer
  listeners: Array<(data: DataPack<unknown>) => void>
  constructor(codec: Codec<DataPack<unknown>>) {
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
  onData(callback: (data: DataPack<unknown>) => void) {
    this.listeners.push(callback);
  }
  async send(data: DataPack<unknown>) {
    const buffer = this.codec.encode(data);
    const buffers = asChunks(buffer);
    for (const chunk of buffers) {
      process.stdout.write(chunk);
    }
  }
}
