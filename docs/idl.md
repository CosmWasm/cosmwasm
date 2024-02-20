# CosmWasm IDL v1.0.0

The CosmWasm IDL (Interface Description Language) is a format for describing the
interface of a smart contract, meant to be consumed by generic clients. This
allows those clients to interact with CosmWasm contracts without having any
prior information about API endpoints.

If you have a smart contract generated from the usual
[template](https://github.com/CosmWasm/cw-template), you should be able to get
an IDL file for it by simply running `cargo schema`.

An example consumer of these files is
[`ts-codegen`](https://github.com/CosmWasm/ts-codegen).

The IDL's only representation is currently JSON-based.

Currently, the IDL format uses [JSON schemas](https://json-schema.org/) heavily
for defining messages and their responses, but provides some metadata and
structure to tie them together.

## An example

The following is an overview with the JSON schemas removed. The full file can be
found
[here](https://github.com/CosmWasm/cosmwasm/blob/v1.5.3/contracts/hackatom/schema/hackatom.json).

```json
{
  "contract_name": "hackatom",
  "contract_version": "0.0.0",
  "idl_version": "1.0.0",
  "instantiate": *JSON_SCHEMA_FOR_INSTANTIATE*,
  "execute": *JSON_SCHEMA_FOR_EXECUTE*,
  "query": *JSON_SCHEMA_FOR_QUERY*,
  "migrate": *JSON_SCHEMA_FOR_MIGRATE*,
  "sudo": *JSON_SCHEMA_FOR_SUDO*,
  "responses": {
    "get_int": *JSON_SCHEMA_FOR_RESPONSE_TO_GET_INT_QUERY*,
    "other_balance": *JSON_SCHEMA_FOR_RESPONSE_TO_OTHER_BALANCE_QUERY*,
  }
}
```

## Fields

### _contract_name_, _contract_version_

Contract metadata. The name is not currently guaranteed to be unique.

### _idl_version_

The version of the IDL format itself. This number adheres to
[Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html).

Using this version number, a client is advised to validate that the IDL files
they're trying to parse are backwards compatible with the IDL version the client
was developed against.

For example, if you're developing a client against the `1.1.0` version of this
spec, this client could accept IDL files for which
`1.1.0 <= idl_version < 2.0.0` is true.

Clients are expected to accept (and ignore) unknown fields. If new fields are
added to the IDL format, this might be considered a backwards compatible change.

### _instantiate_, _execute_, _query_, _migrate_, _sudo_

These are standard entrypoints a smart contract might have. Under these fields,
an IDL file will directly embed a JSON schema for messages that could be passed
to that particular entrypoint.

Out of these, `instantiate` is the only mandatory field every smart contract is
expected to set. The rest of them are optional and might not appear in every IDL
file, meaning the smart contract does not have those entrypoints.

### _responses_

The `responses` field is currently a dictionary mapping of query names to their
response types. The response types are described by embedded JSON schema
objects.

### JSON Schema version

TODO
