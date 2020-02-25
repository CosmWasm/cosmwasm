export function toUtf8(source: string): Uint8Array {
  const buffer = String.UTF8.encode(source);
  return Uint8Array.wrap(buffer);
}

export function fromUtf8(encoded: Uint8Array): string {
  return String.UTF8.decode(encoded.buffer, false);
}
