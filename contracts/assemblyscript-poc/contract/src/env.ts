// Function declatations in this file compile to imports called
// `env.foobar` where `foobar` is the function name.

/* eslint-disable @typescript-eslint/camelcase */

export declare function log(messagePtr: usize): void;

/**
 * Canonicalizes an human readable address
 *
 * @see https://www.cosmwasm.com/docs/intro/addresses
 *
 * @param sourcePtr The human address input
 * @param destinationPtr The canonical address output
 */
export declare function canonicalize_address(sourcePtr: usize, destinationPtr: usize): i32;
