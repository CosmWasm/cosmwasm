use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};

use ibc_reflect::msg::{
    AcknowledgementMsg, BalancesResponse, DispatchResponse, ExecuteMsg, InitMsg, PacketMsg,
    QueryMsg, WhoAmIResponse,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(InitMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(PacketMsg), &out_dir);
    export_schema_with_title(
        &mut schema_for!(AcknowledgementMsg<BalancesResponse>),
        &out_dir,
        "AcknowledgementMsgBalances",
    );
    export_schema_with_title(
        &mut schema_for!(AcknowledgementMsg<DispatchResponse>),
        &out_dir,
        "AcknowledgementMsgDispatch",
    );
    export_schema_with_title(
        &mut schema_for!(AcknowledgementMsg<WhoAmIResponse>),
        &out_dir,
        "AcknowledgementMsgWhoAmI",
    );
}
