# cw721-simple-medadata-without-custom-msg

Example using Custom Metadata extension but not Custom messages. <br>
You have to define your own messages(wrapping base messages) and route them appropriately. <br>

```rust
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    DefaultCw721ExecuteMsg(Box<Cw721ExecuteMsg<Extension, Empty>>),
    ValidHello {},
    InvalidHello {},
}
```

Route default type of message to cw721_execute, and handle your custom message by appropriately routing to custom functions. <br>
If this kind of approach is more desirable, generic type of Custom message extension would be deleted.

```rust
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::DefaultCw721ExecuteMsg(msg) => {
            let res = cw721_execute::<Extension, _, _, _>(deps, env, info, *msg);
            match res {
                Ok(res) => Ok(res),
                Err(err) => Err(ContractError::Cw721ContractError(err)),
            }
        }
        ExecuteMsg::ValidHello {} => valid_hello(),
        ExecuteMsg::InvalidHello {} => invalid_hello(),
    }
}
```