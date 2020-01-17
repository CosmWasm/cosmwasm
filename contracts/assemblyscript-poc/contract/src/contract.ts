import { Encoding } from "./utils";

export function query(): Uint8Array {
  return Encoding.toUtf8("{\"balance\":\"22\"}");
}
