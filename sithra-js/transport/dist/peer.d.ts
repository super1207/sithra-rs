import { DataPack } from ".";
import { Codec } from "./codec";
export declare class Peer {
    codec: Codec<DataPack<unknown>>;
    buffer: Buffer;
    listeners: Array<(data: DataPack<unknown>) => void>;
    constructor(codec: Codec<DataPack<unknown>>);
    onData(callback: (data: DataPack<unknown>) => void): void;
    send(data: DataPack<unknown>): Promise<void>;
}
//# sourceMappingURL=peer.d.ts.map