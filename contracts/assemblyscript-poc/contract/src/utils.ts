
export function getDataPtr(arr: Uint8Array): usize {
  return changetype<usize>(arr.buffer) + arr.byteOffset;
}

export class Encoding {
  public static toUtf8(source: string): Uint8Array {
    const destination = new Uint8Array(String.UTF8.byteLength(source, false));
    const sourcePtr = changetype<usize>(String.UTF8.encode(source, false));
    memory.copy(getDataPtr(destination), sourcePtr, String.UTF8.byteLength(source, false));
    return destination;
  }
}
