# cw721-simple-medadata

Example using cw721-simple-base with metadata extension and custom ExecuteMsg, QueryMsg, ContractError extension. <br>
Define your own type for custom extension, pass them to entry point functions.

```rust
// see: https://docs.opensea.io/docs/metadata-standards
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}
```

Route your custom message to your self designed function when the message is ExecuteMsg::Extension type and use '_' for the basic messages.
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
```