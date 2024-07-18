# IBC Callbacks Contract

This is a simple contract to demonstrate [IBC Callbacks]. It sends an ICS-20 transfer
message to a remote chain and writes to storage which callbacks were called. This
can then be queried using the `CallbackStats` query.

[ibc callbacks]:
  https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-008-app-caller-cbs.md
