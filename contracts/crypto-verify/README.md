# Crypto Verify Contract

This is a simple contract to demonstrate cryptographic signature verification from contracts.

ECDSA Secp256k1 parameters are currently supported. Ed25519 support is upcoming.

## Formats

Input formats are serialized byte slices for Message / Message Hash, Signature, and Public Key.

### secp256k1:

- Hash: SHA-256 hashed value (32 bytes).
- Signature:  Serialized signature, in Cosmos format (64 bytes). Ethereum DER needs to to be converted.
- Public Key:  Compressed (33 bytes) or uncompressed (65 bytes) serialized public key.

Output is a boolean value indicating if verification succeeded or not.


## Remarks

In case of an error (wrong or unsupported inputs), the current (mock) implementation panics. This can be easily debugged,
and optimizes for the production use case, where there is nothing the contract developer should or could handle.
