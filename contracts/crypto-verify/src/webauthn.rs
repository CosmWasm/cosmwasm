//! Adapted from <https://github.com/daimo-eth/p256-verifier/blob/master/test/WebAuthn.t.sol>

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use cosmwasm_std::{Api, StdResult};
use p256::{ecdsa::Signature, elliptic_curve::sec1::FromEncodedPoint, EncodedPoint, PublicKey};
use sha2::{digest::generic_array::GenericArray, Digest, Sha256};

#[allow(clippy::too_many_arguments)]
pub fn verify(
    api: &dyn Api,
    authenticator_data: &[u8],
    client_data_json: &str,
    challenge: &[u8],
    x: &[u8],
    y: &[u8],
    r: &[u8],
    s: &[u8],
) -> StdResult<bool> {
    // We are making a lot of assumptions here about the coordinates, such as:
    //
    // - the length of the encoded bytes being correct
    // - the point being an element of the curve
    // - the conversion from the encoded coordinate to an affine point succeeding
    // - the affine point actually being a valid public key
    // - the signature could actually exist like this for a secp256r1 ECDSA key
    //
    // In production this should have proper error handling
    let point = EncodedPoint::from_affine_coordinates(x.into(), y.into(), false);
    let public_key = PublicKey::from_encoded_point(&point).unwrap();
    let signature = Signature::from_scalars(
        GenericArray::clone_from_slice(r),
        GenericArray::clone_from_slice(s),
    )
    .unwrap();

    // This is missing some checks of some bit flags
    if authenticator_data.len() < 37 {
        return Ok(false);
    }

    // Is this an assertion?
    if !client_data_json.contains("webauthn.get") {
        return Ok(false);
    }

    // Does the challenge belong to the client data?
    let b64_challenge = URL_SAFE_NO_PAD.encode(challenge);
    if !client_data_json.contains(b64_challenge.as_str()) {
        return Ok(false);
    }

    // Verify :D
    let mut hasher = Sha256::new();
    hasher.update(authenticator_data);
    hasher.update(Sha256::digest(client_data_json));
    let hash = hasher.finalize();

    api.secp256r1_verify(&hash, &signature.to_bytes(), &public_key.to_sec1_bytes())
        .map_err(Into::into)
}
