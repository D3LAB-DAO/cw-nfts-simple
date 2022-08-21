# Cw721-simple

---

Cw721 spec-based package to write contract in cw20 style, totally same logic with cw721-base. <br>
Supports flexible contract extension such as metadata extension, custom contract wrapper. <br>



## Implementation

---

If you don't need any metadata extension or custom error, just forward entry points to functions under cw721-simple::contract

```rust
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    cw721_execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cw721_query::<Extension, Empty>(deps, env, msg)
}
```

With metadata extension and custom error, pass your custom types to execute, query functions. <br>

Define your own extension, pass to execute, query functions as parameter of generic types. <br>


```rust
pub type Extension = Option<Metadata>;
pub type MetaMsgExtension = MetaMessage;
```

Extension is type for Metadata of MintMsg, Meta
```rust
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<Extension, MetaMsgExtension>,
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