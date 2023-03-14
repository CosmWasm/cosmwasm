## CosmWasm message types

CosmWasm uses JSON for sending data from the host to the Wasm contract and
results out of the Wasm contract. Such JSON messages are created in the client,
typically some JavaScript-based application. There the usage of JSON feels very
natural for developers. However, JSON has significant limitations such as the
lack of a native binary type and inconsistent support for integers > 53 bit. For
this reason, the CosmWasm standard library `cosmwasm-std` ships types that
ensure good user experience in JSON. The following table shows both standard
Rust types as well as `cosmwasm_std` types and how they are encoded in JSON.

| Rust type           | JSON type[^1]                    | Example                                                                               | Note                                                                                                                                                                                   |
| ------------------- | -------------------------------- | ------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| bool                | `true` or `false`                | `true`                                                                                |                                                                                                                                                                                        |
| u32/i32             | number                           | `123`                                                                                 |                                                                                                                                                                                        |
| u64/i64             | number                           | `123456`                                                                              | Supported in Rust and Go. Other implementations (`jq`, `JavaScript`) do not support the full uint64/int64 range.                                                                       |
| u128/i128           | string                           | `"340282366920938463463374607431768211455", "-2766523308300312711084346401884294402"` | 🚫 Strongly discouraged because the JSON type in serde-json-wasm is wrong and will change. See [Dev Note #4: u128/i128 serialization][dev-note-4].                                     |
| usize/isize         | number                           | `123456`                                                                              | 🚫 Don't use this type because it has a different size in unit tests (64 bit) and Wasm (32 bit). Also it tends to issue float instructions such that the contracts cannot be uploaded. |
| String              | string                           | `"foo"`                                                                               |
| &str                | string                           | `"foo"`                                                                               | 🚫 Unsuppored since message types must be owned (DeserializeOwned)                                                                                                                     |
| Option\<T\>         | `null` or JSON type of `T`       | `null`, `{"foo":12}`                                                                  |                                                                                                                                                                                        |
| Vec\<T\>            | array of JSON type of `T`        | `["one", "two", "three"]` (Vec\<String\>), `[true, false]` (Vec\<bool\>)              |
| Vec\<u8\>           | array of numbers from 0 to 255   | `[187, 61, 11, 250]`                                                                  | ⚠️ Discouraged as this encoding is not as compact as it can be. See `Binary`.                                                                                                          |
| struct MyType { … } | object                           | `{"foo":12}`                                                                          |                                                                                                                                                                                        |
| [Uint64]            | string containing number         | `"1234321"`                                                                           | Used to support full uint64 range in all implementations                                                                                                                               |
| [Uint128]           | string containing number         | `"1234321"`                                                                           |                                                                                                                                                                                        |
| [Uint256]           | string containing number         | `"1234321"`                                                                           |                                                                                                                                                                                        |
| [Uint512]           | string containing number         | `"1234321"`                                                                           |                                                                                                                                                                                        |
| [Decimal]           | string containing decimal number | `"55.6584"`                                                                           |                                                                                                                                                                                        |
| [Decimal256]        | string containing decimal number | `"55.6584"`                                                                           |                                                                                                                                                                                        |
| [Binary]            | string containing base64 data    | `"MTIzCg=="`                                                                          |                                                                                                                                                                                        |
| [HexBinary]         | string containing hex data       | `"b5d7d24e428c"`                                                                      |                                                                                                                                                                                        |

[uint64]: https://docs.rs/cosmwasm-std/1.1.1/cosmwasm_std/struct.Uint64.html
[uint128]: https://docs.rs/cosmwasm-std/1.1.1/cosmwasm_std/struct.Uint128.html
[uint256]: https://docs.rs/cosmwasm-std/1.1.1/cosmwasm_std/struct.Uint256.html
[uint512]: https://docs.rs/cosmwasm-std/1.1.1/cosmwasm_std/struct.Uint512.html
[decimal]: https://docs.rs/cosmwasm-std/1.1.1/cosmwasm_std/struct.Decimal.html
[decimal256]:
  https://docs.rs/cosmwasm-std/1.1.1/cosmwasm_std/struct.Decimal256.html
[binary]: https://docs.rs/cosmwasm-std/1.1.1/cosmwasm_std/struct.Binary.html
[hexbinary]:
  https://docs.rs/cosmwasm-std/1.1.1/cosmwasm_std/struct.HexBinary.html
[dev-note-4]:
  https://medium.com/cosmwasm/dev-note-4-u128-i128-serialization-in-cosmwasm-90cb76784d44

[^1]: https://www.json.org/
