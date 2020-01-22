import { canonicalize } from "./cosmwasm";
import { JsonObject } from "./encoding/json";
import { equalUint8Array } from "./encoding/memory";
import { Encoding } from "./utils";

export function query(msg: JsonObject): Uint8Array {
  if (msg.has("balance")) {
    const address = canonicalize(
      msg
        .get("balance")
        .asObject()
        .get("address")
        .asString()
        .toString(),
    );

    let balance: string;

    if (equalUint8Array(address, canonicalize("addr4321"))) balance = "22";
    else balance = "0";

    return Encoding.toUtf8('{"balance":"' + balance + '"}');
  } else {
    throw new Error("Unsupported query method");
  }
}
