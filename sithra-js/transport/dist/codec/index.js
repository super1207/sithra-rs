import { encode as encode$1, decode } from '@msgpack/msgpack';

function encode(data) {
    let dataRaw = encode$1(data);
    const totalLength = 4 + dataRaw.length;
    const buffer = new ArrayBuffer(totalLength);
    const view = new DataView(buffer);
    view.setUint32(0, dataRaw.length, false);
    const result = new Uint8Array(buffer);
    result.set(dataRaw, 4);
    return result;
}
function decodeFromRaw(buffer) {
    return decode(buffer);
}
function tryDecodeFromRawWithLength(length, buffer) {
    if (buffer.byteLength < length) {
        return null;
    }
    const data = buffer.subarray(0, length);
    return [decodeFromRaw(data), buffer.subarray(length)];
}
function tryDecodeFromRaw(buffer) {
    var _a;
    if (buffer.byteLength < 4) {
        return null;
    }
    const length = buffer.readUInt32BE(0);
    return (_a = tryDecodeFromRawWithLength(length, buffer)) !== null && _a !== void 0 ? _a : length;
}
class DataPackCodec {
    constructor() {
        this.deBuffer = Buffer.from([]);
        this.enBuffer = Buffer.from([]);
        this.dataLength = null;
    }
    decode(chunk) {
        var _a;
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
        let [data, remainingBuffer] = (_a = tryDecodeFromRawWithLength(this.dataLength, this.deBuffer)) !== null && _a !== void 0 ? _a : [null, this.deBuffer];
        this.deBuffer = remainingBuffer;
        this.dataLength = null;
        return data;
    }
    encode(data) {
        return Buffer.from(encode(data));
    }
}

export { DataPackCodec, decodeFromRaw, encode, tryDecodeFromRaw, tryDecodeFromRawWithLength };
//# sourceMappingURL=index.js.map
