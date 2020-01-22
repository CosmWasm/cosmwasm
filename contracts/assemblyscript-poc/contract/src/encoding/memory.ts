/** If you make this public, please write unit tests */
function equalArrayBuffer(a: ArrayBuffer, b: ArrayBuffer): bool {
  if (a === b) return true; // same object optimization

  if (a.byteLength !== b.byteLength) return false;

  const viewA = new DataView(a);
  const viewB = new DataView(b);
  for (let i = 0; i < viewA.byteLength; i++) {
    if (viewA.getInt8(i) !== viewB.getInt8(i)) return false;
  }

  return true;
}

export function equalUint8Array(a: Uint8Array, b: Uint8Array): bool {
  return equalArrayBuffer(a.buffer, b.buffer);
}
