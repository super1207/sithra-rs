export function initStdio(): void {
  if (typeof process.stdin.setRawMode === 'function') {
    process.stdin.setRawMode(true);
  }
  process.stdin.resume();
}

export function asChunks(data: Buffer): Array<Buffer> {
  const chunks: Buffer[] = [];
  const chunkSize = 1024;

  for (let i = 0; i < data.length; i += chunkSize) {
    const end = Math.min(i + chunkSize, data.length);
    chunks.push(data.subarray(i, end));
  }

  return chunks;
}
