use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas};
use cosmwasm_std::Empty;
use cw721_simple::msg::{ExecuteMsg, InstantiateMsg, MintMsg, MinterResponse, QueryMsg};
use schemars::schema_for;
use std::env::current_dir;
use std::fs::create_dir_all;

type Extension = Option<Empty>;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("../schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema_with_title(
        &schema_for!(ExecuteMsg<Extension, Empty>),
        &out_dir,
        "ExecuteMsg",
    );
    export_schema_with_title(&schema_for!(MintMsg<Extension>), &out_dir, "MintMsg");
    export_schema_with_title(&schema_for!(QueryMsg<Extension>), &out_dir, "QueryMsg");
    export_schema(&schema_for!(MinterResponse), &out_dir);
}
