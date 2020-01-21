export function getDataPtr(arr: Uint8Array): usize {
  return changetype<usize>(arr.buffer) + arr.byteOffset;
}

export class Encoding {
  public static toUtf8(source: string): Uint8Array {
    const buffer = String.UTF8.encode(source);

    // Workaround for https://github.com/AssemblyScript/assemblyscript/issues/1066
    if (buffer.byteLength === 0) return new Uint8Array(0);

    return Uint8Array.wrap(buffer);
  }

  public static fromUtf8(encoded: Uint8Array): string {
    return String.UTF8.decode(encoded.buffer, false);
  }
}
