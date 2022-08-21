# Cw721-simple

---

Cw721 spec-based package to write contract in cw20 style, totally same logic with cw721-base. <br>
Supports flexible contract extension such as metadata extension, custom contract wrapper. <br>
Not stand-alone compilable because it's designed only for extension.


## Implementation

---

If you don't need any metadata extension or custom error, just forward entry points to functions under cw721-simple::contract. <br>
With metadata extension and custom error, pass your custom types to execute, query functions. <br>

```rust
pub type Extension = Option<Metadata>;
```

```rust
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<Extension, CustomExtensionMsg>,
) -> Result<Response, ContractError<CustomError>> {
    match msg {
        ExecuteMsg::Extension { msg } => handle_custom_msg(msg),
        _ => cw721_execute(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cw721_query::<Extension, Empty>(deps, env, msg)
}
```

type Extension is for generic type of ExecuteMsg::MintMsg which is for metadata extension, CustomExtensionMsg for ExecuteMsg::Extension <br>
Each types must implement specific traits
* Metadata extension (which is corresponding to Extension) - Serialize, Deserialize, Clone
* Custom execute msg (which is corresponding to CustomExtensionMsg) - Serialize, Deserialize, Clone
* Custom query response - Serialize, Deserialize, Clone
* Custom contract error - Debug, PartialEq, Error
* Custom submsg - CustomMsg



