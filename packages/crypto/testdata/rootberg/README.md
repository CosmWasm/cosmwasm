# Rootberg test data

This folder contains test vectors from
[Project Rootberg](https://github.com/bleichenbacher-daniel/Rooterberg) to
increase the test coverage of public key recovery implementations.

This test data is used by integration tests in `test/rootberg_*.rs`.

## Update

To ensure integrity of the files and update them to the latest version, run this
from the repo root:

```sh
(cd packages/crypto/testdata/rootberg \
  && curl -sSL https://github.com/bleichenbacher-daniel/Rooterberg/raw/main/ecdsa/ecdsa_secp256k1_keccak256_raw.json > ecdsa_secp256k1_keccak256_raw.json \
  && curl -sSL https://github.com/bleichenbacher-daniel/Rooterberg/raw/main/ecdsa/ecdsa_secp256k1_sha_256_raw.json > ecdsa_secp256k1_sha_256_raw.json \
  && curl -sSL https://github.com/bleichenbacher-daniel/Rooterberg/raw/main/ecdsa/ecdsa_secp256r1_keccak256_raw.json > ecdsa_secp256r1_keccak256_raw.json \
  && curl -sSL https://github.com/bleichenbacher-daniel/Rooterberg/raw/main/ecdsa/ecdsa_secp256r1_sha_256_raw.json > ecdsa_secp256r1_sha_256_raw.json \
)
```
