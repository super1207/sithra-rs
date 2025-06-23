function initStdio() {
    if (typeof process.stdin.setRawMode === 'function') {
        process.stdin.setRawMode(true);
    }
    process.stdin.resume();
}
function asChunks(data) {
    const chunks = [];
    const chunkSize = 1024;
    for (let i = 0; i < data.length; i += chunkSize) {
        const end = Math.min(i + chunkSize, data.length);
        chunks.push(data.subarray(i, end));
    }
    return chunks;
}

export { asChunks, initStdio };
//# sourceMappingURL=util.js.map
