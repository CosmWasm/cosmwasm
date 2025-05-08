mod exports;
mod imports;
mod memory; // Used by exports and imports only. This assumes pointers are 32 bit long, which makes it untestable on dev machines.
mod panic;

#[cfg(feature = "cosmwasm_2_2")]
pub use exports::do_migrate_with_info;
pub use exports::{
    do_execute, do_ibc_destination_callback, do_ibc_source_callback, do_instantiate, do_migrate,
    do_query, do_reply, do_sudo,
};
#[cfg(feature = "ibc2")]
pub use exports::{do_ibc2_packet_receive, do_ibc2_packet_timeout};
#[cfg(feature = "stargate")]
pub use exports::{
    do_ibc_channel_close, do_ibc_channel_connect, do_ibc_channel_open, do_ibc_packet_ack,
    do_ibc_packet_receive, do_ibc_packet_timeout,
};
