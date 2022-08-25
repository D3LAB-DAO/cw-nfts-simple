# cw721-simple

Basic example using cw721-simple-base without any extension. <br>
You can use it just simply giving Empty for generic types.

```rust
type Extension = Option<Empty>;
```

```rust
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<Extension, Empty>,
) -> Result<Response, ContractError> {
    cw721_execute(deps, env, info, msg)
}
```