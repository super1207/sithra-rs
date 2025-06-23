import { __awaiter } from '/home/zokute/Work/sithra/sithra-rs/sithra-js/node_modules/tslib/tslib.es6.js';
import { initStdio, asChunks } from './util.js';

class Peer {
    constructor(codec) {
        this.codec = codec;
        this.buffer = Buffer.alloc(0);
        this.listeners = [];
        initStdio();
        process.stdin.on("data", (data) => {
            let decoded = codec.decode(data);
            if (decoded) {
                this.listeners.forEach(listener => listener(decoded));
            }
        });
    }
    onData(callback) {
        this.listeners.push(callback);
    }
    send(data) {
        return __awaiter(this, void 0, void 0, function* () {
            const buffer = this.codec.encode(data);
            const buffers = asChunks(buffer);
            for (const chunk of buffers) {
                process.stdout.write(chunk);
            }
        });
    }
}

export { Peer };
//# sourceMappingURL=peer.js.map
