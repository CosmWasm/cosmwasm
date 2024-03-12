# Wycheproof test data

This folder contains test vectors from
[Project Wycheproof](https://github.com/google/wycheproof) to increase the test
coverage of signature verification implementations.

This test data is used by integration tests in `test/wycheproof_*.rs`.

## Update

To ensure integrity of the files and update them to the latest version, run this
from the repo root:

```sh
(cd packages/crypto/testdata/wycheproof \
  && curl -sSL https://github.com/google/wycheproof/raw/master/testvectors_v1/ecdsa_secp256r1_sha256_test.json > ecdsa_secp256r1_sha256_test.json \
  && curl -sSL https://github.com/google/wycheproof/raw/master/testvectors_v1/ecdsa_secp256r1_sha512_test.json > ecdsa_secp256r1_sha512_test.json \
  && curl -sSL https://github.com/google/wycheproof/raw/master/testvectors_v1/ecdsa_secp256r1_sha3_256_test.json > ecdsa_secp256r1_sha3_256_test.json \
  && curl -sSL https://github.com/google/wycheproof/raw/master/testvectors_v1/ecdsa_secp256r1_sha3_512_test.json > ecdsa_secp256r1_sha3_512_test.json \
  && curl -sSL https://github.com/google/wycheproof/raw/master/testvectors_v1/ecdsa_secp256k1_sha256_test.json > ecdsa_secp256k1_sha256_test.json \
  && curl -sSL https://github.com/google/wycheproof/raw/master/testvectors_v1/ecdsa_secp256k1_sha512_test.json > ecdsa_secp256k1_sha512_test.json \
  && curl -sSL https://github.com/google/wycheproof/raw/master/testvectors_v1/ecdsa_secp256k1_sha3_256_test.json > ecdsa_secp256k1_sha3_256_test.json \
  && curl -sSL https://github.com/google/wycheproof/raw/master/testvectors_v1/ecdsa_secp256k1_sha3_512_test.json > ecdsa_secp256k1_sha3_512_test.json \
  )
```
