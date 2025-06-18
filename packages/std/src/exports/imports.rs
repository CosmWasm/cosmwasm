use alloc::vec::Vec;
use core::ptr;

#[cfg(feature = "iterator")]
use super::memory::get_optional_region_address;
use super::memory::{Owned, Region};
use crate::import_helpers::{from_high_half, from_low_half};
#[cfg(feature = "iterator")]
use crate::iterator::{Order, Record};
use crate::results::SystemResult;
#[cfg(feature = "iterator")]
use crate::sections::decode_sections2;
use crate::sections::encode_sections;
use crate::serde::from_json;
use crate::traits::{Api, Querier, QuerierResult, Storage};
use crate::{Addr, CanonicalAddr};
#[cfg(feature = "cosmwasm_2_1")]
use crate::{AggregationError, HashFunction, PairingEqualityError};
use crate::{
    RecoverPubkeyError, StdError, StdErrorKind, StdResult, SystemError, VerificationError,
};

/// An upper bound for typical canonical address lengths (e.g. 20 in Cosmos SDK/Ethereum or 32 in Nano/Substrate)
const CANONICAL_ADDRESS_BUFFER_LENGTH: usize = 64;
/// An upper bound for typical human readable address formats (e.g. 42 for Ethereum hex addresses or 90 for bech32)
const HUMAN_ADDRESS_BUFFER_LENGTH: usize = 90;

// This interface will compile into required Wasm imports.
// A complete documentation those functions is available in the VM that provides them:
// https://github.com/CosmWasm/cosmwasm/blob/v1.0.0-beta/packages/vm/src/instance.rs#L89-L206
extern "C" {

    fn abort(source_ptr: u32);

    fn db_read(key: u32) -> u32;
    fn db_write(key: u32, value: u32);
    fn db_remove(key: u32);

    // scan creates an iterator, which can be read by consecutive next() calls
    #[cfg(feature = "iterator")]
    fn db_scan(start_ptr: u32, end_ptr: u32, order: i32) -> u32;
    #[cfg(feature = "iterator")]
    fn db_next(iterator_id: u32) -> u32;
    #[cfg(all(feature = "iterator", feature = "cosmwasm_1_4"))]
    fn db_next_key(iterator_id: u32) -> u32;
    #[cfg(all(feature = "iterator", feature = "cosmwasm_1_4"))]
    fn db_next_value(iterator_id: u32) -> u32;

    fn addr_validate(source_ptr: u32) -> u32;
    fn addr_canonicalize(source_ptr: u32, destination_ptr: u32) -> u32;
    fn addr_humanize(source_ptr: u32, destination_ptr: u32) -> u32;

    #[cfg(feature = "cosmwasm_2_1")]
    fn bls12_381_aggregate_g1(g1s_ptr: u32, out_ptr: u32) -> u32;

    #[cfg(feature = "cosmwasm_2_1")]
    fn bls12_381_aggregate_g2(g2s_ptr: u32, out_ptr: u32) -> u32;

    #[cfg(feature = "cosmwasm_2_1")]
    fn bls12_381_pairing_equality(ps_ptr: u32, qs_ptr: u32, r_ptr: u32, s_ptr: u32) -> u32;

    #[cfg(feature = "cosmwasm_2_1")]
    fn bls12_381_hash_to_g1(hash_function: u32, msg_ptr: u32, dst_ptr: u32, out_ptr: u32) -> u32;

    #[cfg(feature = "cosmwasm_2_1")]
    fn bls12_381_hash_to_g2(hash_function: u32, msg_ptr: u32, dst_ptr: u32, out_ptr: u32) -> u32;

    /// Verifies message hashes against a signature with a public key, using the
    /// secp256k1 ECDSA parametrization.
    /// Returns 0 on verification success, 1 on verification failure, and values
    /// greater than 1 in case of error.
    fn secp256k1_verify(message_hash_ptr: u32, signature_ptr: u32, public_key_ptr: u32) -> u32;

    fn secp256k1_recover_pubkey(
        message_hash_ptr: u32,
        signature_ptr: u32,
        recovery_param: u32,
    ) -> u64;

    /// Verifies message hashes against a signature with a public key, using the
    /// secp256r1 ECDSA parametrization.
    /// Returns 0 on verification success, 1 on verification failure, and values
    /// greater than 1 in case of error.
    #[cfg(feature = "cosmwasm_2_1")]
    fn secp256r1_verify(message_hash_ptr: u32, signature_ptr: u32, public_key_ptr: u32) -> u32;

    #[cfg(feature = "cosmwasm_2_1")]
    fn secp256r1_recover_pubkey(
        message_hash_ptr: u32,
        signature_ptr: u32,
        recovery_param: u32,
    ) -> u64;

    /// Verifies a message against a signature with a public key, using the
    /// ed25519 EdDSA scheme.
    /// Returns 0 on verification success, 1 on verification failure, and values
    /// greater than 1 in case of error.
    fn ed25519_verify(message_ptr: u32, signature_ptr: u32, public_key_ptr: u32) -> u32;

    /// Verifies a batch of messages against a batch of signatures and public keys, using the
    /// ed25519 EdDSA scheme.
    /// Returns 0 on verification success, 1 on verification failure, and values
    /// greater than 1 in case of error.
    fn ed25519_batch_verify(messages_ptr: u32, signatures_ptr: u32, public_keys_ptr: u32) -> u32;

    /// Writes a debug message (UFT-8 encoded) to the host for debugging purposes.
    /// The host is free to log or process this in any way it considers appropriate.
    /// In production environments it is expected that those messages are discarded.
    fn debug(source_ptr: u32);

    /// Executes a query on the chain (import). Not to be confused with the
    /// query export, which queries the state of the contract.
    fn query_chain(request: u32) -> u32;
}

/// A stateless convenience wrapper around database imports provided by the VM.
/// This cannot be cloned as it would not copy any data. If you need to clone this, it indicates a flaw in your logic.
pub struct ExternalStorage {}

impl ExternalStorage {
    pub fn new() -> ExternalStorage {
        ExternalStorage {}
    }
}

impl Storage for ExternalStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let key = Region::from_slice(key);
        let key_ptr = key.as_ptr() as u32;

        let read = unsafe { db_read(key_ptr) };
        if read == 0 {
            // key does not exist in external storage
            return None;
        }

        let value_ptr = read as *mut Region<Owned>;
        let data = unsafe { Region::from_heap_ptr(ptr::NonNull::new(value_ptr).unwrap()) };

        Some(data.into_vec())
    }

    fn set(&mut self, key: &[u8], value: &[u8]) {
        if value.is_empty() {
            panic!("TL;DR: Value must not be empty in Storage::set but in most cases you can use Storage::remove instead. Long story: Getting empty values from storage is not well supported at the moment. Some of our internal interfaces cannot differentiate between a non-existent key and an empty value. Right now, you cannot rely on the behaviour of empty values. To protect you from trouble later on, we stop here. Sorry for the inconvenience! We highly welcome you to contribute to CosmWasm, making this more solid one way or the other.");
        }

        let key = Region::from_slice(key);
        let key_ptr = key.as_ptr() as u32;

        let value = Region::from_slice(value);
        let value_ptr = value.as_ptr() as u32;

        unsafe { db_write(key_ptr, value_ptr) };
    }

    fn remove(&mut self, key: &[u8]) {
        let key = Region::from_slice(key);
        let key_ptr = key.as_ptr() as u32;

        unsafe { db_remove(key_ptr) };
    }

    #[cfg(feature = "iterator")]
    fn range(
        &self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Record>> {
        let iterator_id = create_iter(start, end, order);
        let iter = ExternalIterator { iterator_id };
        Box::new(iter)
    }

    #[cfg(all(feature = "cosmwasm_1_4", feature = "iterator"))]
    fn range_keys<'a>(
        &'a self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Vec<u8>> + 'a> {
        let iterator_id = create_iter(start, end, order);
        let iter = ExternalPartialIterator {
            iterator_id,
            partial_type: PartialType::Keys,
        };
        Box::new(iter)
    }

    #[cfg(all(feature = "cosmwasm_1_4", feature = "iterator"))]
    fn range_values<'a>(
        &'a self,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
        order: Order,
    ) -> Box<dyn Iterator<Item = Vec<u8>> + 'a> {
        let iterator_id = create_iter(start, end, order);
        let iter = ExternalPartialIterator {
            iterator_id,
            partial_type: PartialType::Values,
        };
        Box::new(iter)
    }
}

#[cfg(feature = "iterator")]
fn create_iter(start: Option<&[u8]>, end: Option<&[u8]>, order: Order) -> u32 {
    // There is lots of gotchas on turning options into regions for FFI, thus this design
    // See: https://github.com/CosmWasm/cosmwasm/pull/509
    let start_region = start.map(Region::from_slice);
    let end_region = end.map(Region::from_slice);
    let start_region_addr = get_optional_region_address(&start_region.as_ref());
    let end_region_addr = get_optional_region_address(&end_region.as_ref());
    unsafe { db_scan(start_region_addr, end_region_addr, order as i32) }
}

#[cfg(all(feature = "cosmwasm_1_4", feature = "iterator"))]
enum PartialType {
    Keys,
    Values,
}

/// ExternalPartialIterator makes a call out to `next_key` or `next_value`
/// depending on its `partial_type`.
/// Compared to `ExternalIterator`, it allows iterating only over the keys or
/// values instead of both.
#[cfg(all(feature = "cosmwasm_1_4", feature = "iterator"))]
struct ExternalPartialIterator {
    iterator_id: u32,
    partial_type: PartialType,
}

#[cfg(all(feature = "cosmwasm_1_4", feature = "iterator"))]
impl Iterator for ExternalPartialIterator {
    type Item = Vec<u8>;

    /// The default implementation calls `next` repeatedly,
    /// which we can do a little more efficiently by using `db_next_key` instead.
    /// It is used by `skip`, so it allows cheaper skipping.
    #[cfg(feature = "cosmwasm_1_4")]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        skip_iter(self.iterator_id, n);
        self.next()
    }

    fn next(&mut self) -> Option<Self::Item> {
        // here we differentiate between the two types
        let next_result = match self.partial_type {
            PartialType::Keys => unsafe { db_next_key(self.iterator_id) },
            PartialType::Values => unsafe { db_next_value(self.iterator_id) },
        };

        if next_result == 0 {
            // iterator is done
            return None;
        }

        let data_region = next_result as *mut Region<Owned>;
        let data = unsafe { Region::from_heap_ptr(ptr::NonNull::new(data_region).unwrap()) };

        Some(data.into_vec())
    }
}

#[cfg(feature = "iterator")]
/// ExternalIterator makes a call out to next.
/// We use the pointer to differentiate between multiple open iterators.
struct ExternalIterator {
    iterator_id: u32,
}

#[cfg(feature = "iterator")]
impl Iterator for ExternalIterator {
    type Item = Record;

    /// The default implementation calls `next` repeatedly,
    /// which we can do a little more efficiently by using `db_next_key` instead.
    /// It is used by `skip`, so it allows cheaper skipping.
    #[cfg(feature = "cosmwasm_1_4")]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        skip_iter(self.iterator_id, n);
        self.next()
    }

    fn next(&mut self) -> Option<Self::Item> {
        let next_result = unsafe { db_next(self.iterator_id) };
        let kv_region_ptr = next_result as *mut Region<Owned>;
        let kv = unsafe { Region::from_heap_ptr(ptr::NonNull::new(kv_region_ptr).unwrap()) };

        let (key, value) = decode_sections2(kv.into_vec());

        if key.len() == 0 {
            None
        } else {
            Some((key, value))
        }
    }
}

/// Helper function to skip `count` elements of an iterator.
#[cfg(all(feature = "iterator", feature = "cosmwasm_1_4"))]
fn skip_iter(iter_id: u32, count: usize) {
    for _ in 0..count {
        let region = unsafe { db_next_key(iter_id) };
        if region == 0 {
            // early return
            return;
        }

        // just deallocate the region
        unsafe { Region::from_heap_ptr(ptr::NonNull::new(region as *mut Region<Owned>).unwrap()) };
    }
}

/// A stateless convenience wrapper around imports provided by the VM
#[derive(Copy, Clone)]
pub struct ExternalApi {}

impl ExternalApi {
    pub fn new() -> ExternalApi {
        ExternalApi {}
    }
}

impl Api for ExternalApi {
    fn addr_validate(&self, input: &str) -> StdResult<Addr> {
        let input_bytes = input.as_bytes();
        if input_bytes.len() > 256 {
            // See MAX_LENGTH_HUMAN_ADDRESS in the VM.
            // In this case, the VM will refuse to read the input from the contract.
            // Stop here to allow handling the error in the contract.
            return Err(
                StdError::msg("input too long for addr_validate").with_kind(StdErrorKind::Parsing)
            );
        }
        let source = Region::from_slice(input_bytes);
        let source_ptr = source.as_ptr() as u32;

        let result = unsafe { addr_validate(source_ptr) };
        if result != 0 {
            let error =
                unsafe { consume_string_region_written_by_vm(result as *mut Region<Owned>) };
            return Err(
                StdError::msg(format_args!("addr_validate errored: {}", error))
                    .with_kind(StdErrorKind::Parsing),
            );
        }

        Ok(Addr::unchecked(input))
    }

    fn addr_canonicalize(&self, input: &str) -> StdResult<CanonicalAddr> {
        let input_bytes = input.as_bytes();
        if input_bytes.len() > 256 {
            // See MAX_LENGTH_HUMAN_ADDRESS in the VM.
            // In this case, the VM will refuse to read the input from the contract.
            // Stop here to allow handling the error in the contract.
            return Err(StdError::msg("input too long for addr_canonicalize")
                .with_kind(StdErrorKind::Parsing));
        }
        let send = Region::from_slice(input_bytes);
        let send_ptr = send.as_ptr() as u32;
        let canon = Region::with_capacity(CANONICAL_ADDRESS_BUFFER_LENGTH);

        let result = unsafe { addr_canonicalize(send_ptr, canon.as_ptr() as u32) };
        if result != 0 {
            let error =
                unsafe { consume_string_region_written_by_vm(result as *mut Region<Owned>) };
            return Err(
                StdError::msg(format_args!("addr_canonicalize errored: {}", error))
                    .with_kind(StdErrorKind::Parsing),
            );
        }

        Ok(CanonicalAddr::from(canon.into_vec()))
    }

    fn addr_humanize(&self, canonical: &CanonicalAddr) -> StdResult<Addr> {
        let send = Region::from_slice(canonical.as_slice());
        let send_ptr = send.as_ptr() as u32;
        let human = Region::with_capacity(HUMAN_ADDRESS_BUFFER_LENGTH);

        let result = unsafe { addr_humanize(send_ptr, human.as_ptr() as u32) };
        if result != 0 {
            let error =
                unsafe { consume_string_region_written_by_vm(result as *mut Region<Owned>) };
            return Err(
                StdError::msg(format_args!("addr_humanize errored: {}", error))
                    .with_kind(StdErrorKind::Encoding),
            );
        }

        let address = unsafe { String::from_utf8_unchecked(human.into_vec()) };
        Ok(Addr::unchecked(address))
    }

    #[cfg(feature = "cosmwasm_2_1")]
    fn bls12_381_aggregate_g1(&self, g1s: &[u8]) -> Result<[u8; 48], VerificationError> {
        let point = [0_u8; 48];

        let send = Region::from_slice(g1s);
        let send_ptr = send.as_ptr() as u32;

        let out = Region::from_slice(&point);
        let out_ptr = out.as_ptr() as u32;
        let result = unsafe { bls12_381_aggregate_g1(send_ptr, out_ptr) };
        match result {
            0 => Ok(point),
            8 => Err(VerificationError::InvalidPoint),
            16 => Err(VerificationError::Aggregation {
                source: AggregationError::Empty,
            }),
            17 => Err(VerificationError::Aggregation {
                source: AggregationError::NotMultiple,
            }),
            error_code => Err(VerificationError::unknown_err(error_code)),
        }
    }

    #[cfg(feature = "cosmwasm_2_1")]
    fn bls12_381_aggregate_g2(&self, g2s: &[u8]) -> Result<[u8; 96], VerificationError> {
        let point = [0_u8; 96];

        let send = Region::from_slice(g2s);
        let send_ptr = send.as_ptr() as u32;

        let out = Region::from_slice(&point);
        let out_ptr = out.as_ptr() as u32;
        let result = unsafe { bls12_381_aggregate_g2(send_ptr, out_ptr) };
        match result {
            0 => Ok(point),
            8 => Err(VerificationError::InvalidPoint),
            14 => Err(VerificationError::Aggregation {
                source: AggregationError::Empty,
            }),
            15 => Err(VerificationError::Aggregation {
                source: AggregationError::NotMultiple,
            }),
            error_code => Err(VerificationError::unknown_err(error_code)),
        }
    }

    #[cfg(feature = "cosmwasm_2_1")]
    fn bls12_381_pairing_equality(
        &self,
        ps: &[u8],
        qs: &[u8],
        r: &[u8],
        s: &[u8],
    ) -> Result<bool, VerificationError> {
        let send_ps = Region::from_slice(ps);
        let send_qs = Region::from_slice(qs);
        let send_r = Region::from_slice(r);
        let send_s = Region::from_slice(s);

        let send_ps_ptr = send_ps.as_ptr() as u32;
        let send_qs_ptr = send_qs.as_ptr() as u32;
        let send_r_ptr = send_r.as_ptr() as u32;
        let send_s_ptr = send_s.as_ptr() as u32;

        let result =
            unsafe { bls12_381_pairing_equality(send_ps_ptr, send_qs_ptr, send_r_ptr, send_s_ptr) };
        match result {
            0 => Ok(true),
            1 => Ok(false),
            8 => Err(VerificationError::InvalidPoint),
            11 => Err(VerificationError::PairingEquality {
                source: PairingEqualityError::NotMultipleG1,
            }),
            12 => Err(VerificationError::PairingEquality {
                source: PairingEqualityError::NotMultipleG2,
            }),
            13 => Err(VerificationError::PairingEquality {
                source: PairingEqualityError::UnequalPointAmount,
            }),
            error_code => Err(VerificationError::unknown_err(error_code)),
        }
    }

    #[cfg(feature = "cosmwasm_2_1")]
    fn bls12_381_hash_to_g1(
        &self,
        hash_function: HashFunction,
        msg: &[u8],
        dst: &[u8],
    ) -> Result<[u8; 48], VerificationError> {
        let point = [0_u8; 48];

        let send_msg = Region::from_slice(msg);
        let send_msg_ptr = send_msg.as_ptr() as u32;

        let send_dst = Region::from_slice(dst);
        let send_dst_ptr = send_dst.as_ptr() as u32;

        let out = Region::from_slice(&point);
        let out_ptr = out.as_ptr() as u32;
        let result = unsafe {
            bls12_381_hash_to_g1(hash_function as u32, send_msg_ptr, send_dst_ptr, out_ptr)
        };

        match result {
            0 => Ok(point),
            9 => Err(VerificationError::UnknownHashFunction),
            error_code => Err(VerificationError::unknown_err(error_code)),
        }
    }

    #[cfg(feature = "cosmwasm_2_1")]
    fn bls12_381_hash_to_g2(
        &self,
        hash_function: HashFunction,
        msg: &[u8],
        dst: &[u8],
    ) -> Result<[u8; 96], VerificationError> {
        let point = [0_u8; 96];

        let send_msg = Region::from_slice(msg);
        let send_msg_ptr = send_msg.as_ptr() as u32;

        let send_dst = Region::from_slice(dst);
        let send_dst_ptr = send_dst.as_ptr() as u32;

        let out = Region::from_slice(&point);
        let out_ptr = out.as_ptr() as u32;
        let result = unsafe {
            bls12_381_hash_to_g2(hash_function as u32, send_msg_ptr, send_dst_ptr, out_ptr)
        };

        match result {
            0 => Ok(point),
            9 => Err(VerificationError::UnknownHashFunction),
            error_code => Err(VerificationError::unknown_err(error_code)),
        }
    }

    fn secp256k1_verify(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        let hash_send = Region::from_slice(message_hash);
        let hash_send_ptr = hash_send.as_ptr() as u32;
        let sig_send = Region::from_slice(signature);
        let sig_send_ptr = sig_send.as_ptr() as u32;
        let pubkey_send = Region::from_slice(public_key);
        let pubkey_send_ptr = pubkey_send.as_ptr() as u32;

        let result = unsafe { secp256k1_verify(hash_send_ptr, sig_send_ptr, pubkey_send_ptr) };
        match result {
            0 => Ok(true),
            1 => Ok(false),
            2 => panic!("MessageTooLong must not happen. This is a bug in the VM."),
            3 => Err(VerificationError::InvalidHashFormat),
            4 => Err(VerificationError::InvalidSignatureFormat),
            5 => Err(VerificationError::InvalidPubkeyFormat),
            10 => Err(VerificationError::GenericErr),
            error_code => Err(VerificationError::unknown_err(error_code)),
        }
    }

    fn secp256k1_recover_pubkey(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        recover_param: u8,
    ) -> Result<Vec<u8>, RecoverPubkeyError> {
        let hash_send = Region::from_slice(message_hash);
        let hash_send_ptr = hash_send.as_ptr() as u32;
        let sig_send = Region::from_slice(signature);
        let sig_send_ptr = sig_send.as_ptr() as u32;

        let result =
            unsafe { secp256k1_recover_pubkey(hash_send_ptr, sig_send_ptr, recover_param.into()) };
        let error_code = from_high_half(result);
        let pubkey_ptr = from_low_half(result);
        match error_code {
            0 => {
                let pubkey = unsafe {
                    Region::from_heap_ptr(
                        ptr::NonNull::new(pubkey_ptr as *mut Region<Owned>).unwrap(),
                    )
                    .into_vec()
                };
                Ok(pubkey)
            }
            2 => panic!("MessageTooLong must not happen. This is a bug in the VM."),
            3 => Err(RecoverPubkeyError::InvalidHashFormat),
            4 => Err(RecoverPubkeyError::InvalidSignatureFormat),
            6 => Err(RecoverPubkeyError::InvalidRecoveryParam),
            error_code => Err(RecoverPubkeyError::unknown_err(error_code)),
        }
    }

    #[cfg(feature = "cosmwasm_2_1")]
    fn secp256r1_verify(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        let hash_send = Region::from_slice(message_hash);
        let hash_send_ptr = hash_send.as_ptr() as u32;
        let sig_send = Region::from_slice(signature);
        let sig_send_ptr = sig_send.as_ptr() as u32;
        let pubkey_send = Region::from_slice(public_key);
        let pubkey_send_ptr = pubkey_send.as_ptr() as u32;

        let result = unsafe { secp256r1_verify(hash_send_ptr, sig_send_ptr, pubkey_send_ptr) };
        match result {
            0 => Ok(true),
            1 => Ok(false),
            2 => panic!("MessageTooLong must not happen. This is a bug in the VM."),
            3 => Err(VerificationError::InvalidHashFormat),
            4 => Err(VerificationError::InvalidSignatureFormat),
            5 => Err(VerificationError::InvalidPubkeyFormat),
            10 => Err(VerificationError::GenericErr),
            error_code => Err(VerificationError::unknown_err(error_code)),
        }
    }

    #[cfg(feature = "cosmwasm_2_1")]
    fn secp256r1_recover_pubkey(
        &self,
        message_hash: &[u8],
        signature: &[u8],
        recover_param: u8,
    ) -> Result<Vec<u8>, RecoverPubkeyError> {
        let hash_send = Region::from_slice(message_hash);
        let hash_send_ptr = hash_send.as_ptr() as u32;
        let sig_send = Region::from_slice(signature);
        let sig_send_ptr = sig_send.as_ptr() as u32;

        let result =
            unsafe { secp256r1_recover_pubkey(hash_send_ptr, sig_send_ptr, recover_param.into()) };
        let error_code = from_high_half(result);
        let pubkey_ptr = from_low_half(result);
        match error_code {
            0 => {
                let pubkey = unsafe {
                    Region::from_heap_ptr(
                        ptr::NonNull::new(pubkey_ptr as *mut Region<Owned>).unwrap(),
                    )
                    .into_vec()
                };
                Ok(pubkey)
            }
            2 => panic!("MessageTooLong must not happen. This is a bug in the VM."),
            3 => Err(RecoverPubkeyError::InvalidHashFormat),
            4 => Err(RecoverPubkeyError::InvalidSignatureFormat),
            6 => Err(RecoverPubkeyError::InvalidRecoveryParam),
            error_code => Err(RecoverPubkeyError::unknown_err(error_code)),
        }
    }

    fn ed25519_verify(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        let msg_send = Region::from_slice(message);
        let msg_send_ptr = msg_send.as_ptr() as u32;
        let sig_send = Region::from_slice(signature);
        let sig_send_ptr = sig_send.as_ptr() as u32;
        let pubkey_send = Region::from_slice(public_key);
        let pubkey_send_ptr = pubkey_send.as_ptr() as u32;

        let result = unsafe { ed25519_verify(msg_send_ptr, sig_send_ptr, pubkey_send_ptr) };
        match result {
            0 => Ok(true),
            1 => Ok(false),
            2 => panic!("Error code 2 unused since CosmWasm 0.15. This is a bug in the VM."),
            3 => panic!("InvalidHashFormat must not happen. This is a bug in the VM."),
            4 => Err(VerificationError::InvalidSignatureFormat),
            5 => Err(VerificationError::InvalidPubkeyFormat),
            10 => Err(VerificationError::GenericErr),
            error_code => Err(VerificationError::unknown_err(error_code)),
        }
    }

    fn ed25519_batch_verify(
        &self,
        messages: &[&[u8]],
        signatures: &[&[u8]],
        public_keys: &[&[u8]],
    ) -> Result<bool, VerificationError> {
        let msgs_encoded = encode_sections(messages);
        let msgs_send = Region::from_vec(msgs_encoded);
        let msgs_send_ptr = msgs_send.as_ptr() as u32;

        let sigs_encoded = encode_sections(signatures);
        let sig_sends = Region::from_vec(sigs_encoded);
        let sigs_send_ptr = sig_sends.as_ptr() as u32;

        let pubkeys_encoded = encode_sections(public_keys);
        let pubkeys_send = Region::from_vec(pubkeys_encoded);
        let pubkeys_send_ptr = pubkeys_send.as_ptr() as u32;

        let result =
            unsafe { ed25519_batch_verify(msgs_send_ptr, sigs_send_ptr, pubkeys_send_ptr) };
        match result {
            0 => Ok(true),
            1 => Ok(false),
            2 => panic!("Error code 2 unused since CosmWasm 0.15. This is a bug in the VM."),
            3 => panic!("InvalidHashFormat must not happen. This is a bug in the VM."),
            4 => Err(VerificationError::InvalidSignatureFormat),
            5 => Err(VerificationError::InvalidPubkeyFormat),
            10 => Err(VerificationError::GenericErr),
            error_code => Err(VerificationError::unknown_err(error_code)),
        }
    }

    fn debug(&self, message: &str) {
        // keep the boxes in scope, so we free it at the end (don't cast to pointers same line as Region::from_slice)
        let region = Region::from_slice(message.as_bytes());
        let region_ptr = region.as_ptr() as u32;
        unsafe { debug(region_ptr) };
    }
}

/// Takes a pointer to a Region and reads the data into a String.
/// This is for trusted string sources only.
unsafe fn consume_string_region_written_by_vm(from: *mut Region<Owned>) -> String {
    let data = Region::from_heap_ptr(ptr::NonNull::new(from).unwrap()).into_vec();
    // We trust the VM/chain to return correct UTF-8, so let's save some gas
    String::from_utf8_unchecked(data)
}

/// A stateless convenience wrapper around imports provided by the VM
pub struct ExternalQuerier {}

impl ExternalQuerier {
    pub fn new() -> ExternalQuerier {
        ExternalQuerier {}
    }
}

impl Querier for ExternalQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let req = Region::from_slice(bin_request);
        let request_ptr = req.as_ptr() as u32;

        let response_ptr = unsafe { query_chain(request_ptr) };
        let response = unsafe {
            Region::from_heap_ptr(ptr::NonNull::new(response_ptr as *mut Region<Owned>).unwrap())
                .into_vec()
        };

        from_json(&response).unwrap_or_else(|parsing_err| {
            SystemResult::Err(SystemError::InvalidResponse {
                error: parsing_err.to_string(),
                response: response.into(),
            })
        })
    }
}

pub fn handle_panic(message: &str) {
    let region = Region::from_slice(message.as_bytes());
    let region_ptr = region.as_ptr() as u32;
    unsafe { abort(region_ptr) };
}
