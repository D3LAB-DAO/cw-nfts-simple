use std::env::current_dir;
use std::fs::create_dir_all;
use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas};
use cosmwasm_std::Empty;
use schemars::schema_for;
use cw721_simple_base::msg::{InstantiateMsg, ExecuteMsg, MintMsg, QueryMsg, MinterResponse};

type Extension = Option<Empty>;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("../schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema_with_title(&schema_for!(ExecuteMsg<Extension>), &out_dir, "ExecuteMsg");
    export_schema_with_title(&schema_for!(MintMsg<Extension>), &out_dir, "MintMsg");
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(MinterResponse), &out_dir);
}
