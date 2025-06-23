import { __awaiter } from '/home/zokute/Work/sithra/sithra-rs/sithra-js/node_modules/tslib/tslib.es6.js';
import { DataPackCodec } from './codec/index.js';
import { Peer } from './peer.js';

let codec = new DataPackCodec();
let peer = new Peer(codec);
let data = {
    path: "/test",
    correlation: "01JY43C3GKC60NXBY9ZTS1RRFF",
    payload: 'Hello, World!'
};
function sleep(ms) {
    return __awaiter(this, void 0, void 0, function* () {
        return new Promise(resolve => setTimeout(resolve, ms));
    });
}
peer.onData((data) => {
    console.error("recv msg:", JSON.stringify(data));
});
(() => __awaiter(void 0, void 0, void 0, function* () {
    for (;;) {
        yield sleep(1000);
        peer.send(data);
    }
}))();
//# sourceMappingURL=test.js.map
