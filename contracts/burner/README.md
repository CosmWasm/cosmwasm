# Burner Contract

This is a simple contract to demonstrate using migrations to
shutdown (or "burn") contracts using the migration feature
added in CosmWasm 0.9.

This contract cannot be installed directly (via `init`), but is only
designed to be used for `migrate`. When migrating any existing
contract to this burner contract, we delete all storage and
send all bank tokens to a specified address, doing a basic
cleanup of the contract.

You can use this contract as-is, or fork it and customize it
more if you want to do more detailed cleanup.
