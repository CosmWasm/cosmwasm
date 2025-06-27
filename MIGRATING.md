# Migrating Contracts

This guide explains what is needed to upgrade contracts when migrating over
major releases of `cosmwasm`. Note that you can also view the
[complete CHANGELOG](./CHANGELOG.md) to understand the differences.

## 1.5.x -> 2.0.x

- Update `cosmwasm-*` dependencies in Cargo.toml (skip the ones you don't use):

  ```toml
  [dependencies]
  cosmwasm-std = "2.0.0"
  # ...

  [dev-dependencies]
  cosmwasm-schema = "2.0.0"
  cosmwasm-vm = "2.0.0"
  # ...
  ```

  If you were using cosmwasm-std's `ibc3` feature, you can remove it, as it is
  the default now. Depending on your usage, you might have to enable the
  `stargate` feature instead, since it was previously implied by `ibc3`.

  Also remove any uses of the `backtraces` feature. You can use a
  `RUST_BACKTRACE=1` environment variable for this now.

  If you were using `cosmwasm-std` with `default-features = false`, you probably
  want to enable the `std` feature now, as we might move certain existing
  functionality to that feature in the future to support no_std environments:

  ```diff
  -cosmwasm-std = { version = "2.0.0", default-features = false, features = [...] }
  +cosmwasm-std = { version = "2.0.0", default-features = false, features = ["std", ...] }
  ```

- If you want to use a feature that is only available on CosmWasm 2.0+ chains,
  use this feature:

  ```diff
  -cosmwasm-std = { version = "1.4.0", features = ["stargate"] }
  +cosmwasm-std = { version = "1.4.0", features = ["stargate", "cosmwasm_2_0"] }
  ```

  Please note that `cosmwasm_2_0` implies `cosmwasm_1_4`, `cosmwasm_1_3` and so
  on, so there is no need to set multiple.

- `ContractInfoResponse::new` now takes all fields of the response as
  parameters:

  ```diff
  -ContractInfoResponse::new(code_id, creator)
  +ContractInfoResponse::new(code_id, creator, admin, pinned, ibc_port)
  ```

  Please note that, in the future, this function signature can change between
  minor versions.

- Replace all uses of `SubMsgExecutionResponse` with `SubMsgResponse`.
- Replace all uses of `PartialEq<&str> for Addr` with `PartialEq<Addr> for Addr`
  like this:

  ```diff
  -if addr == "admin" {
  -    // ...
  -}
  +let admin = deps.api.addr_validate("admin")?;
  +if addr == admin {
  +  // ...
  +}
  ```

  If you really want to compare the string representation (e.g. in tests), you
  can use `Addr::as_str`:

  ```diff
  -assert_eq!(addr, "admin");
  +assert_eq!(addr.as_str(), "admin");
  ```

  But keep in mind that this is case sensitive (while addresses are not).

- Replace all uses of `Mul<Decimal> for Uint128` and
  `Mul<Decimal256> for Uint256` with `Uint{128,256}::mul_floor`:

  ```diff
  -Uint128::new(123456) * Decimal::percent(1);
  +Uint128::new(123456).mul_floor(Decimal::percent(1));
  ```

- When calling `Coin::new`, you now have to explicitly specify the integer type:

  ```diff
  -Coin::new(1234, "uatom")
  +Coin::new(1234u128, "uatom")
  ```

- When creating a `Binary` or `Size` instance from an inner value, you now have
  to explicitly call `new`:

  ```diff
  -Binary(vec![1u8])
  +Binary::new(vec![1u8])
  ```

- When accessing the inner value of a `CanonicalAddr` or `Binary`, use
  `as_slice` instead:

  ```diff
  -&canonical_addr.0
  +canonical_addr.as_slice()
  ```

- If you use any `u128` or `i128` in storage or message types, replace them with
  `Uint128` and `Int128` respectively to preserve the current serialization.
  Failing to do this will result in deserialization errors!

  ```diff
  #[cw_serde]
  struct MyStorage {
  -  a: u128,
  -  b: i128,
  +  a: Uint128,
  +  b: Int128,
  }
  const map: Map<u128, MyStorage> = Map::new("map");

  -const item: Item<u128> = Item::new("item");
  +const item: Item<Uint128> = Item::new("item");
  ```

- Replace all uses of `IbcReceiveResponse::set_ack` and
  `IbcReceiveResponse::default` with calls to `IbcReceiveResponse::new`:

  ```diff
  -Ok(IbcReceiveResponse::new().set_ack(b"{}"))
  +Ok(IbcReceiveResponse::new(b"{}"))

  -Ok(IbcReceiveResponse::default())
  +Ok(IbcReceiveResponse::new(b""))
  ```

- Replace all uses of `CosmosMsg::Stargate` with `CosmosMsg::Any`:

  ```diff
  -CosmosMsg::Stargate { type_url, value }
  +CosmosMsg::Any(AnyMsg { type_url, value })
  ```

- Replace all direct construction of `StdError` with the use of the
  corresponding constructor:

  ```diff
  -StdError::GenericErr { msg }
  +StdError::generic_err(msg)
  ```

- Replace addresses in unit tests with valid Bech32 addresses. This has to be
  done for all addresses that are validated or canonicalized during the test or
  within the contract. The easiest way to do this is by using
  `MockApi::addr_make`. It generates a Bech32 address from any string:

  ```diff
  -let msg = InstantiateMsg {
  -    verifier: "verifier".to_string(),
  -    beneficiary: "beneficiary".to_string(),
  -};
  +let msg = InstantiateMsg {
  +    verifier: deps.api.addr_make("verifier").to_string(),
  +    beneficiary: deps.api.addr_make("beneficiary").to_string(),
  +};
  ```

- Replace addresses in integration tests using `cosmwasm-vm` with valid Bech32
  addresses. This has to be done for all addresses that are validated or
  canonicalized during the test or within the contract. The easiest way to do
  this is by using `MockApi::addr_make`. It generates a Bech32 address from any
  string:

  ```diff
  -let msg = InstantiateMsg {
  -    verifier: "verifier".to_string(),
  -    beneficiary: "beneficiary".to_string(),
  -};
  +let msg = InstantiateMsg {
  +    verifier: instance.api().addr_make("verifier").to_string(),
  +    beneficiary: instance.api().addr_make("beneficiary").to_string(),
  +};
  ```

- The `update_balance`, `set_denom_metadata`, `set_withdraw_address`,
  `set_withdraw_addresses` and `clear_withdraw_addresses` functions were removed
  from the `MockQuerier`. Use the newly exposed modules to access them directly:

  ```diff
  -querier.update_balance("addr", coins(1000, "ATOM"));
  +querier.bank.update_balance("addr", coins(1000, "ATOM"));
  -querier.set_withdraw_address("delegator", "withdrawer");
  +querier.distribution.set_withdraw_address("delegator", "withdrawer");
  -querier.update_staking(denom, &[], &[]);
  +querier.staking.update(denom, &[], &[]);
  -querier.update_ibc(port_id, &[]);
  +querier.ibc.update(port_id, &[]);
  ```

- If you were using `QueryRequest::Stargate`, you might want to enable the
  `cosmwasm_2_0` cargo feature and migrate to `QueryRequest::Grpc` instead.
  While the stargate query sometimes returns protobuf-encoded data and sometimes
  JSON encoded data, depending on the chain, the gRPC query always returns
  protobuf-encoded data.

  ```diff
  -deps.querier.query(&QueryRequest::Stargate {
  -    path: "/service.Path/ServiceMethod".to_string(),
  -    data: Binary::new(b"DATA"),
  -})?;
  +deps.querier.query(&QueryRequest::Grpc(GrpcQuery {
  +    path: "/service.Path/ServiceMethod".to_string(),
  +    data: Binary::new(b"DATA"),
  +}))?;
  ```

- A new `payload` field allows you to send arbitrary data from the original
  contract into the `reply`. If you construct `SubMsg` manually, add the
  `payload` field:

  ```diff
   SubMsg {
       id: 12,
  +    payload: Binary::default(),
       msg: my_bank_send,
       gas_limit: Some(12345u64),
       reply_on: ReplyOn::Always,
   },
  ```

  or with data:

  ```diff
   SubMsg {
       id: 12,
  +    payload: Binary::new(vec![9, 8, 7, 6, 5]),
       msg: my_bank_send,
       gas_limit: Some(12345u64),
       reply_on: ReplyOn::Always,
   },
  ```

  If you use a constructor function, you can set the payload as follows:

  ```diff
   SubMsg::new(BankMsg::Send {
     to_address: payout,
     amount: coins(123456u128,"gold")
   })
  +.with_payload(vec![9, 8, 7, 6, 5])
  ```

  The payload data will then be available in the new field `Reply.payload` in
  the `reply` entry point. This functionality is an optional addition introduced
  in 2.0. To keep the CosmWasm 1.x behavior, just set payload to
  `Binary::default()`.

- In test code, replace calls to `mock_info` with `message_info`. This takes a
  `&Addr` as the first argument which you get by using owned `Addr` in the test
  bodies.

## 1.4.x -> 1.5.0

- Update `cosmwasm-*` dependencies in Cargo.toml (skip the ones you don't use):

  ```
  [dependencies]
  cosmwasm-std = "1.5.0"
  cosmwasm-storage = "1.5.0"
  # ...

  [dev-dependencies]
  cosmwasm-schema = "1.5.0"
  cosmwasm-vm = "1.5.0"
  # ...
  ```

## 1.3.x -> 1.4.0

- Update `cosmwasm-*` dependencies in Cargo.toml (skip the ones you don't use):

  ```
  [dependencies]
  cosmwasm-std = "1.4.0"
  cosmwasm-storage = "1.4.0"
  # ...

  [dev-dependencies]
  cosmwasm-schema = "1.4.0"
  cosmwasm-vm = "1.4.0"
  # ...
  ```

- If you want to use a feature that is only available on CosmWasm 1.4+ chains,
  use this feature:

  ```diff
  -cosmwasm-std = { version = "1.4.0", features = ["stargate"] }
  +cosmwasm-std = { version = "1.4.0", features = ["stargate", "cosmwasm_1_4"] }
  ```

  Please note that `cosmwasm_1_2` implies `cosmwasm_1_1`, and `cosmwasm_1_3`
  implies `cosmwasm_1_2`, and so on, so there is no need to set multiple.

## 1.2.x -> 1.3.0

- Update `cosmwasm-*` dependencies in Cargo.toml (skip the ones you don't use):

  ```
  [dependencies]
  cosmwasm-std = "1.3.0"
  cosmwasm-storage = "1.3.0"
  # ...

  [dev-dependencies]
  cosmwasm-schema = "1.3.0"
  cosmwasm-vm = "1.3.0"
  # ...
  ```

- If you want to use a feature that is only available on CosmWasm 1.3+ chains,
  use this feature:

  ```diff
  -cosmwasm-std = { version = "1.3.0", features = ["stargate"] }
  +cosmwasm-std = { version = "1.3.0", features = ["stargate", "cosmwasm_1_3"] }
  ```

  Please note that `cosmwasm_1_2` implies `cosmwasm_1_1`, and `cosmwasm_1_3`
  implies `cosmwasm_1_2`, and so on, so there is no need to set multiple.

## 1.1.x -> 1.2.0

- Update `cosmwasm-*` dependencies in Cargo.toml (skip the ones you don't use):

  ```
  [dependencies]
  cosmwasm-std = "1.2.0"
  cosmwasm-storage = "1.2.0"
  # ...

  [dev-dependencies]
  cosmwasm-schema = "1.2.0"
  cosmwasm-vm = "1.2.0"
  # ...
  ```

- If you want to use a feature that is only available on CosmWasm 1.2+ chains,
  use this feature:

  ```diff
  -cosmwasm-std = { version = "1.1.0", features = ["stargate"] }
  +cosmwasm-std = { version = "1.1.0", features = ["stargate", "cosmwasm_1_2"] }
  ```

  Please note that `cosmwasm_1_2` implies `cosmwasm_1_1`, so there is no need to
  set both.

- If you use mixed type multiplication between `Uint{64,128,256}` and
  `Decimal{,256}`, check out
  `mul_floor`/`checked_mul_floor`/`mul_ceil`/`checked_mul_ceil`. Mixed type
  arithmetic [will be removed](https://github.com/CosmWasm/cosmwasm/issues/1485)
  at some point.

  ```diff
  let a = Uint128::new(123);
  let b = Decimal::percent(150)

  -let c = a * b;
  +let c = a.mul_floor(b);
  ```

## 1.0.0 -> 1.1.0

- Update `cosmwasm-*` dependencies in Cargo.toml (skip the ones you don't use):

  ```
  [dependencies]
  cosmwasm-std = "1.1.0"
  cosmwasm-storage = "1.1.0"
  # ...

  [dev-dependencies]
  cosmwasm-schema = "1.1.0"
  cosmwasm-vm = "1.1.0"
  # ...
  ```

- There are changes to how we generate schemas, resulting in less boilerplate
  maintenance for smart contract devs. Old contracts will continue working for a
  while, but it's highly recommended to migrate now.

  Your contract should have a `cosmwasm_schema` dependency in its `Cargo.toml`
  file. Move it from `dev-dependencies`to regular `dependencies`.

  ```diff
    [dependencies]
  + cosmwasm-schema = { version = "1.1.0" }
    cosmwasm-std = { version = "1.1.0", features = ["stargate"] }
    cw-storage-plus = { path = "../../packages/storage-plus", version = "0.10.0" }
    schemars = "0.8.1"
    serde = { version = "1.0.103", default-features = false, features = ["derive"] }
    thiserror = { version = "1.0.23" }

    [dev-dependencies]
  - cosmwasm-schema = { version = "1.1.0" }
  ```

  Types you send to the contract and receive back are annotated with a bunch of
  derives and sometimes `serde` annotations. Remove all those attributes and
  replace them with `#[cosmwasm_schema::cw_serde]`.

  ```diff
  + use cosmwasm_schema::{cw_serde, QueryResponses};

    // *snip*

  - #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
  - #[serde(deny_unknown_fields, rename_all = "snake_case")]
  + #[cw_serde]
    pub enum ExecuteMsg {
        Release {},
        Argon2 {
            mem_cost: u32,
            time_cost: u32,
        },
    }
  ```

  Derive `cosmwasm_schema::QueryResponses` for your `QueryMsg` type and annotate
  each query with its return type. This lets the interface description file
  (schema) generation know what return types to include - and therefore, any
  clients relying on the generated schemas will also know how to interpret
  response data from your contract.

  ```diff
    #[cw_serde]
  + #[derive(QueryResponses)]
    pub enum QueryMsg {
  +     #[returns(VerifierResponse)]
        Verifier {},
  +     #[returns(Uint128)]
        Balance { address: String },
    }
  ```

  The boilerplate in `examples/schema.rs` is also replaced with a macro
  invocation. Just give it all the types sent to the contract's entrypoints.
  Skip the ones that are not present in the contract - the only mandatory field
  is `instantiate`.

  ```rust
  use cosmwasm_schema::write_api;

  use hackatom::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SudoMsg};

  fn main() {
      write_api! {
          instantiate: InstantiateMsg,
          query: QueryMsg,
          execute: ExecuteMsg,
          sudo: SudoMsg,
          migrate: MigrateMsg,
      }
  }
  ```

This changes the format of the schemas generated by the contract. They're now in
one structured, unified file (parseable by machines) rather than a bunch of
arbitrary ones.

## 1.0.0-beta -> 1.0.0

- The minimum Rust supported version is 1.56.1. Verify your Rust version is >=
  1.56.1 with: `rustc --version`. Please note that the required Rust version
  changes over time and we have little control over that due to the dependencies
  that are used.

- Simplify `mock_dependencies` calls with empty balance:

  ```diff
       #[test]
       fn instantiate_fails() {
  -        let mut deps = mock_dependencies(&[]);
  +        let mut deps = mock_dependencies();

           let msg = InstantiateMsg {};
           let info = mock_info("creator", &coins(1000, "earth"));
  ```

  Or use the new `mock_dependencies_with_balance` if you need a balance:

  ```diff
       #[test]
       fn migrate_cleans_up_data() {
  -        let mut deps = mock_dependencies(&coins(123456, "gold"));
  +        let mut deps = mock_dependencies_with_balance(&coins(123456, "gold"));

           // store some sample data
           deps.storage.set(b"foo", b"bar");
  ```

- Replace `ContractResult` with `SubMsgResult` in `Reply` handling:

  ```diff
  @@ -35,10 +35,10 @@ pub fn instantiate(
   #[entry_point]
   pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> StdResult<Response> {
       match (reply.id, reply.result) {
  -        (RECEIVE_DISPATCH_ID, ContractResult::Err(err)) => {
  +        (RECEIVE_DISPATCH_ID, SubMsgResult::Err(err)) => {
               Ok(Response::new().set_data(encode_ibc_error(err)))
           }
  -        (INIT_CALLBACK_ID, ContractResult::Ok(response)) => handle_init_callback(deps, response),
  +        (INIT_CALLBACK_ID, SubMsgResult::Ok(response)) => handle_init_callback(deps, response),
           _ => Err(StdError::generic_err("invalid reply id or result")),
       }
   }
  ```

- Replace `SubMsgExecutionResponse` with `SubMsgResponse`:

  ```diff
  @@ -387,7 +384,7 @@ mod tests {
           // fake a reply and ensure this works
           let response = Reply {
               id,
  -            result: SubMsgResult::Ok(SubMsgExecutionResponse {
  +            result: SubMsgResult::Ok(SubMsgResponse {
                   events: fake_events(&account),
                   data: None,
               }),
  ```

## 0.16 -> 1.0.0-beta

- Update CosmWasm dependencies in Cargo.toml (skip the ones you don't use):

  ```
  [dependencies]
  cosmwasm-std = "1.0.0-beta"
  cosmwasm-storage = "1.0.0-beta"
  # ...

  [dev-dependencies]
  cosmwasm-schema = "1.0.0-beta"
  cosmwasm-vm = "1.0.0-beta"
  # ...
  ```

- Use type `Record` instead of `Pair`

  ```rust
  // before
  use cosmwasm_std::Pair;

  // after
  use cosmwasm_std::Record;
  ```

- Replace `cosmwasm_std::create_entry_points!` and
  `cosmwasm_std::create_entry_points_with_migration!` with `#[entry_point]`
  annotations. See the [0.13 -> 0.14 entry](#013---014) where `#[entry_point]`
  was introduced.

- If your chain provides a custom query, add the custom query type as a generic
  argument to `cosmwasm_std::Deps`, `DepsMut`, `OwnedDeps` and `QuerierWrapper`.
  Otherwise, it defaults to `Empty`. E.g.

  ```diff
   #[entry_point]
   pub fn instantiate(
  -    deps: DepsMut,
  +    deps: DepsMut<CyberQueryWrapper>,
       _env: Env,
       info: MessageInfo,
       msg: InstantiateMsg,
  @@ -38,112 +35,95 @@ pub fn instantiate(
   }
  ```

  ```diff
   #[entry_point]
  -pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
  +pub fn query(deps: Deps<CyberQueryWrapper>, _env: Env, msg: QueryMsg) ->   StdResult<Binary> {
       match msg {
  ```

  ```diff
   pub struct CyberQuerier<'a> {
  -    querier: &'a QuerierWrapper<'a>,
  +    querier: &'a QuerierWrapper<'a, CyberQueryWrapper>,
   }

   impl<'a> CyberQuerier<'a> {
  -    pub fn new(querier: &'a QuerierWrapper) -> Self {
  +    pub fn new(querier: &'a QuerierWrapper<'a, CyberQueryWrapper>) -> Self {
           CyberQuerier { querier }
       }
   }
  ```

  Replace `QuerierWrapper::custom_query` with `QuerierWrapper::query` which is
  now fully typed:

  ```diff
  -let res: CyberlinksAmountResponse = self.querier.custom_query(&request.into())?;
  +let res: CyberlinksAmountResponse = self.querier.query(&request.into())?;
  ```

  See https://github.com/cybercongress/cw-cyber/pull/2 for a complete example.

### Integration tests

- Add new `transaction` field to `Env` when creating a custom mock env:

  ```diff
  @@ -19,6 +19,7 @@

   use cosmwasm_std::{
       coins, Addr, BlockInfo, Coin, ContractInfo, Env, MessageInfo, Response, Timestamp,
  +    TransactionInfo,
   };
   use cosmwasm_storage::to_length_prefixed;
   use cosmwasm_vm::testing::{instantiate, mock_info, mock_instance};
  @@ -52,6 +53,7 @@ fn mock_env_info_height(signer: &str, sent: &[Coin], height: u64, time: u64) ->
           contract: ContractInfo {
               address: Addr::unchecked(MOCK_CONTRACT_ADDR),
           },
  +        transaction: Some(TransactionInfo { index: 3 }),
       };
       let info = mock_info(signer, sent);
       return (env, info);
  ```

- Gas usage increases by a factor of approximately 150_000. Adapt your tests
  accordingly.

## 0.15 -> 0.16

- Update CosmWasm dependencies in Cargo.toml (skip the ones you don't use):

  ```
  [dependencies]
  cosmwasm-std = "0.16.0"
  cosmwasm-storage = "0.16.0"
  # ...

  [dev-dependencies]
  cosmwasm-schema = "0.16.0"
  cosmwasm-vm = "0.16.0"
  # ...
  ```

- The `attr` function now accepts arguments that implement `Into<String>` rather
  than `ToString`. This means that "stringly" types like `&str` are still
  accepted, but others (like numbers or booleans) have to be explicitly
  converted to strings; you can use the `to_string` method (from the
  `std::string::ToString` trait) for that.

  ```diff
    let steal_funds = true;
  - attr("steal_funds", steal_funds),
  + attr("steal_funds", steal_funds.to_string()),
  ```

  It also means that `&&str` is no longer accepted.

- The `iterator` feature in `cosmwasm-std`, `cosmwasm-vm` and `cosmwasm-storage`
  is now enabled by default. If you want to use it, you don't have to explicitly
  enable it anymore.

  If you don't want to use it, you **have to** disable default features when
  depending on `cosmwasm-std`. Example:

  ```diff
  - cosmwasm-std = { version = "0.15.0" }
  + cosmwasm-std = { version = "0.16.0", default-features = false }
  ```

- The `Event::attr` setter has been renamed to `Event::add_attribute` - this is
  for consistency with other types, like `Response`.

  ```diff
  - let event = Event::new("ibc").attr("channel", "connect");
  + let event = Event::new("ibc").add_attribute("channel", "connect");
  ```

- `Response` can no longer be built using a struct literal. Please use
  `Response::new` as well as relevant
  [builder-style setters](https://github.com/CosmWasm/cosmwasm/blob/402e3281ff5bc1cd7b4b3e36c2bb9914f07eaaf6/packages/std/src/results/response.rs#L103-L167)
  to set the data.

  This is a step toward better API stability.

  ```diff
    #[entry_point]
    pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> StdResult<Response> {
        // ...

        let send = BankMsg::Send {
            to_address: msg.payout.clone(),
            amount: balance,
        };
        let data_msg = format!("burnt {} keys", count).into_bytes();

  -     Ok(Response {
  -         messages: vec![SubMsg::new(send)],
  -         attributes: vec![attr("action", "burn"), attr("payout", msg.payout)],
  -         events: vec![],
  -         data: Some(data_msg.into()),
  -     })
  +     Ok(Response::new()
  +         .add_message(send)
  +         .add_attribute("action", "burn")
  +         .add_attribute("payout", msg.payout)
  +         .set_data(data_msg))
    }
  ```

  ```diff
  - Ok(Response {
  -     data: Some((old_size as u32).to_be_bytes().into()),
  -     ..Response::default()
  - })
  + Ok(Response::new().set_data((old_size as u32).to_be_bytes()))
  ```

  ```diff
  - let res = Response {
  -     messages: msgs,
  -     attributes: vec![attr("action", "reflect_subcall")],
  -     events: vec![],
  -     data: None,
  - };
  - Ok(res)
  + Ok(Response::new()
  +     .add_attribute("action", "reflect_subcall")
  +     .add_submessages(msgs))
  ```

- For IBC-enabled contracts only: constructing `IbcReceiveResponse` and
  `IbcBasicResponse` follows the same principles now as `Response` above.

  ```diff
    pub fn ibc_packet_receive(
        deps: DepsMut,
        env: Env,
        msg: IbcPacketReceiveMsg,
    ) -> StdResult<IbcReceiveResponse> {
        // ...

  -     Ok(IbcReceiveResponse {
  -         acknowledgement,
  -         messages: vec![],
  -         attributes: vec![],
  -         events: vec![Event::new("ibc").attr("packet", "receive")],
  -     })
  +     Ok(IbcReceiveResponse::new()
  +         .set_ack(acknowledgement)
  +         .add_event(Event::new("ibc").add_attribute("packet", "receive")))
    }
  ```

- For IBC-enabled contracts only: IBC entry points have different signatures.
  Instead of accepting bare packets, channels and acknowledgements, all of those
  are wrapped in a `Msg` type specific to the given entry point. Channels,
  packets and acknowledgements have to be unpacked from those.

  ```diff
    #[entry_point]
  - pub fn ibc_channel_open(_deps: DepsMut, _env: Env, channel: IbcChannel) -> StdResult<()> {
  + pub fn ibc_channel_open(_deps: DepsMut, _env: Env, msg: IbcChannelOpenMsg) -> StdResult<()> {
  +     let channel = msg.channel();

        // do things
    }
  ```

  ```diff
    #[entry_point]
    pub fn ibc_channel_connect(
        deps: DepsMut,
        env: Env,
  -     channel: IbcChannel,
  +     msg: IbcChannelConnectMsg,
    ) -> StdResult<IbcBasicResponse> {
  +     let channel = msg.channel();

        // do things
    }
  ```

  ```diff
    #[entry_point]
    pub fn ibc_channel_close(
        deps: DepsMut,
        env: Env,
  -     channel: IbcChannel,
  +     msg: IbcChannelCloseMsg,
    ) -> StdResult<IbcBasicResponse> {
  +     let channel = msg.channel();

        // do things
    }
  ```

  ```diff
    #[entry_point]
    pub fn ibc_packet_receive(
        deps: DepsMut,
        env: Env,
  -     packet: IbcPacket,
  +     msg: IbcPacketReceiveMsg,
    ) -> StdResult<IbcReceiveResponse> {
  +     let packet = msg.packet;

        // do things
    }
  ```

  ```diff
    #[entry_point]
    pub fn ibc_packet_receive(
        deps: DepsMut,
        env: Env,
  -     ack: IbcAcknowledgementWithPacket,
  +     msg: IbcPacketReceiveMsg,
    ) -> StdResult<IbcBasicResponse> {
        // They are the same struct just a different name
        let ack = msg;

        // do things
    }
  ```

  ```diff
    #[entry_point]
    pub fn ibc_packet_timeout(
        deps: DepsMut,
        env: Env,
  -     packet: IbcPacket,
  +     msg: IbcPacketTimeoutMsg,
    ) -> StdResult<IbcBasicResponse> {
  +     let packet = msg.packet;

        // do things
    }
  ```

## 0.14 -> 0.15

- Update CosmWasm dependencies in Cargo.toml (skip the ones you don't use):

  ```
  [dependencies]
  cosmwasm-std = "0.15.0"
  cosmwasm-storage = "0.15.0"
  # ...

  [dev-dependencies]
  cosmwasm-schema = "0.15.0"
  cosmwasm-vm = "0.15.0"
  # ...
  ```

- Combine `messages` and `submessages` on the `Response` object. The new format
  uses `messages: Vec<SubMsg<T>>`, so copy `submessages` content, and wrap old
  messages using `SubMsg::new`. Here is how to change messages:

  ```rust
  let send = BankMsg::Send { to_address, amount };

  // before
  let res = Response {
    messages: vec![send.into()],
    ..Response::default()
  }

  // after
  let res = Response {
    messages: vec![SubMsg::new(send)],
    ..Response::default()
  }

  // alternate approach
  let mut res = Response::new();
  res.add_message(send);
  ```

  And here is how to change submessages:

  ```rust
  // before
  let sub_msg = SubMsg {
    id: INIT_CALLBACK_ID,
    msg: msg.into(),
    gas_limit: None,
    reply_on: ReplyOn::Success,
  };
  let res = Response {
    submessages: vec![sub_msg],
    ..Response::default()
  };

  // after
  let msg = SubMsg::reply_on_success(msg, INIT_CALLBACK_ID);
  let res = Response {
    messages: vec![msg],
    ..Response::default()
  };

  // alternate approach
  let msg = SubMsg::reply_on_success(msg, INIT_CALLBACK_ID);
  let mut res = Response::new();
  res.add_submessage(msg);
  ```

  Note that this means you can mix "messages" and "submessages" in any execution
  order. You are no more restricted to doing "submessages" first.

- Rename the `send` field to `funds` whenever constructing a `WasmMsg::Execute`
  or `WasmMsg::Instantiate` value.

  ```diff
    let exec = WasmMsg::Execute {
        contract_addr: coin.address.into(),
        msg: to_binary(&msg)?,
  -     send: vec![],
  +     funds: vec![],
    };
  ```

- `Uint128` field can no longer be constructed using a struct literal. Call
  `Uint128::new` (or `Uint128::zero`) instead.

  ```diff
  - const TOKENS_PER_WEIGHT: Uint128 = Uint128(1_000);
  - const MIN_BOND: Uint128 = Uint128(5_000);
  + const TOKENS_PER_WEIGHT: Uint128 = Uint128::new(1_000);
  + const MIN_BOND: Uint128 = Uint128::new(5_000);
  ```

  ```diff
  - assert_eq!(escrow_balance, Uint128(0));
  + assert_eq!(escrow_balance, Uint128::zero());
  ```

- If constructing a `Response` using struct literal syntax, add the `events`
  field.

  ```diff
    Ok(Response {
        messages: vec![],
        attributes,
  +     events: vec![],
        data: None,
    })
  ```

- For IBC-enabled contracts only: You need to adapt to the new
  `IbcAcknowledgementWithPacket` structure and use the embedded `data` field:

  ```rust
  // before
  pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    ack: IbcAcknowledgement,
  ) -> StdResult<Response> {
    let res: AcknowledgementMsg = from_slice(&ack.acknowledgement)?;
    // ...
  }

  // after
  pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    ack: IbcAcknowledgementWithPacket,
  ) -> StdResult<Response> {
    let res: AcknowledgementMsg = from_slice(&ack.acknowledgement.data)?;
    // ...
  }
  ```

  You also need to update the constructors in test code. Below we show how to do
  so both for JSON data as well as any custom binary format:

  ```rust
  // before (JSON)
  let ack = IbcAcknowledgement {
    acknowledgement: to_binary(&AcknowledgementMsg::Ok(())).unwrap()
    original_packet: packet,
  };

  // after (JSON)
  let ack = IbcAcknowledgementWithPacket {
      acknowledgement: IbcAcknowledgement::encode_json(&AcknowledgementMsg::Ok(())).unwrap(),
      original_packet: packet,
  };

  // before (Custom binary data)
  let acknowledgement = vec![12, 56, 78];
  let ack = IbcAcknowledgement {
    acknowledgement: Binary(acknowledgement),
    original_packet: packet,
  };

  // after (Custom binary data)
  let acknowledgement = vec![12, 56, 78];
  let ack = IbcAcknowledgement {
    acknowledgement: IbcAcknowledgement::new(acknowledgement),
    original_packet: packet,
  };
  ```

## 0.13 -> 0.14

- The minimum Rust supported version for 0.14 is 1.51.0. Verify your Rust
  version is >= 1.51.0 with: `rustc --version`

- Update CosmWasm and schemars dependencies in Cargo.toml (skip the ones you
  don't use):

  ```
  [dependencies]
  cosmwasm-std = "0.14.0"
  cosmwasm-storage = "0.14.0"
  schemars = "0.8.1"
  # ...

  [dev-dependencies]
  cosmwasm-schema = "0.14.0"
  cosmwasm-vm = "0.14.0"
  # ...
  ```

- Rename the `init` entry point to `instantiate`. Also, rename `InitMsg` to
  `InstantiateMsg`.

- Rename the `handle` entry point to `execute`. Also, rename `HandleMsg` to
  `ExecuteMsg`.

- Rename `InitResponse`, `HandleResponse` and `MigrateResponse` to `Response`.
  The old names are still supported (with a deprecation warning), and will be
  removed in the next version. Also, you'll need to add the `submessages` field
  to `Response`.

- Remove `from_address` from `BankMsg::Send`, which is now automatically filled
  with the contract address:

  ```rust
  // before
  ctx.add_message(BankMsg::Send {
      from_address: env.contract.address,
      to_address: to_addr,
      amount: balance,
  });

  // after
  ctx.add_message(BankMsg::Send {
      to_address: to_addr,
      amount: balance,
  });
  ```

- Use the new entry point system. From `lib.rs` remove

  ```rust
  #[cfg(target_arch = "wasm32")]
  cosmwasm_std::create_entry_points!(contract);

  // or

  #[cfg(target_arch = "wasm32")]
  cosmwasm_std::create_entry_points_with_migration!(contract);
  ```

  Then add the macro attribute `#[entry_point]` to your `contract.rs` as
  follows:

  ```rust
  use cosmwasm_std::{entry_point, … };

  // …

  #[entry_point]
  pub fn init(
      _deps: DepsMut,
      _env: Env,
      _info: MessageInfo,
      _msg: InitMsg,
  ) -> StdResult<Response> {
      // …
  }

  #[entry_point]
  pub fn execute(
      _deps: DepsMut,
      _env: Env,
      _info: MessageInfo,
      _msg: ExecuteMsg,
  ) -> StdResult<Response> {
      // …
  }

  // only if you have migrate
  #[entry_point]
  pub fn migrate(
      deps: DepsMut,
      env: Env,
      _info: MessageInfo,
      msg: MigrateMsg,
  ) -> StdResult<Response> {
      // …
  }

  #[entry_point]
  pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<QueryResponse> {
      // …
  }
  ```

- Since `Response` contains a `data` field, converting `Context` into `Response`
  always succeeds.

  ```rust
  // before
  pub fn init(deps: DepsMut, env: Env, info: MessageInfo, msg: InitMsg) -> Result<InitResponse, HackError> {
      // …
      let mut ctx = Context::new();
      ctx.add_attribute("Let the", "hacking begin");
      Ok(ctx.try_into()?)
  }

  // after
  pub fn init(deps: DepsMut, env: Env, info: MessageInfo, msg: InitMsg) -> Result<Response, HackError> {
      // …
      let mut ctx = Context::new();
      ctx.add_attribute("Let the", "hacking begin");
      Ok(ctx.into())
  }
  ```

- Remove the `info: MessageInfo` field from the `migrate` entry point:

  ```rust
  // Before
  pub fn migrate(
      deps: DepsMut,
      env: Env,
      _info: MessageInfo,
      msg: MigrateMsg,
  ) -> StdResult<MigrateResponse> {
    // ...
  }

  // After
  pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> StdResult<Response> {
    // ...
  }
  ```

  `MessageInfo::funds` was always empty since [MsgMigrateContract] does not have
  a funds field. `MessageInfo::sender` should not be needed for authentication
  because the chain checks permissions before calling `migrate`. If the sender's
  address is needed for anything else, this should be expressed as part of the
  migrate message.

  [msgmigratecontract]:
    https://github.com/CosmWasm/wasmd/blob/v0.15.0/x/wasm/internal/types/tx.proto#L86-L96

- Add mutating helper methods to `Response` that can be used instead of creating
  a `Context` that is later converted to a response:

  ```rust
  // before
  pub fn handle_impl(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
      // ...

      // release counter_offer to creator
      let mut ctx = Context::new();
      ctx.add_message(BankMsg::Send {
          to_address: state.creator,
          amount: state.counter_offer,
      });

      // release collateral to sender
      ctx.add_message(BankMsg::Send {
          to_address: state.owner,
          amount: state.collateral,
      });

      // ..

      ctx.add_attribute("action", "execute");
      Ok(ctx.into())
  }


  // after
  pub fn execute_impl(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
      // ...

      // release counter_offer to creator
      let mut resp = Response::new();
      resp.add_message(BankMsg::Send {
          to_address: state.creator,
          amount: state.counter_offer,
      });

      // release collateral to sender
      resp.add_message(BankMsg::Send {
          to_address: state.owner,
          amount: state.collateral,
      });

      // ..

      resp.add_attribute("action", "execute");
      Ok(resp)
  }
  ```

- Use type `Pair` instead of `KV`

  ```rust
  // before
  use cosmwasm_std::KV;

  // after
  use cosmwasm_std::Pair;
  ```

- If necessary, add a wildcard arm to the `match` of now non-exhaustive message
  types `BankMsg`, `BankQuery`, `WasmMsg` and `WasmQuery`.

- `HumanAddr` has been deprecated in favour of simply `String`. It never added
  any significant safety bonus over `String` and was just a marker type. The new
  type `Addr` was created to hold validated addresses. Those can be created via
  `Addr::unchecked`, `Api::addr_validate`, `Api::addr_humanize` and JSON
  deserialization. In order to maintain type safety, deserialization into `Addr`
  must only be done from trusted sources like a contract's state or a query
  response. User inputs must be deserialized into `String`. This new `Addr` type
  makes it easy to use human readable addresses in state:

  With pre-validated `Addr` from `MessageInfo`:

  ```rust
  // before
  pub struct State {
      pub owner: CanonicalAddr,
  }

  let state = State {
      owner: deps.api.canonical_address(&info.sender /* of type HumanAddr */)?,
  };


  // after
  pub struct State {
      pub owner: Addr,
  }
  let state = State {
      owner: info.sender.clone() /* of type Addr */,
  };
  ```

  With user input in `msg`:

  ```rust
  // before
  pub struct State {
      pub verifier: CanonicalAddr,
      pub beneficiary: CanonicalAddr,
      pub funder: CanonicalAddr,
  }

  deps.storage.set(
      CONFIG_KEY,
      &to_vec(&State {
          verifier: deps.api.canonical_address(&msg.verifier /* of type HumanAddr */)?,
          beneficiary: deps.api.canonical_address(&msg.beneficiary /* of type HumanAddr */)?,
          funder: deps.api.canonical_address(&info.sender /* of type HumanAddr */)?,
      })?,
  );

  // after
  pub struct State {
      pub verifier: Addr,
      pub beneficiary: Addr,
      pub funder: Addr,
  }

  deps.storage.set(
      CONFIG_KEY,
      &to_vec(&State {
          verifier: deps.api.addr_validate(&msg.verifier /* of type String */)?,
          beneficiary: deps.api.addr_validate(&msg.beneficiary /* of type String */)?,
          funder: info.sender /* of type Addr */,
      })?,
  );
  ```

  The existing `CanonicalAddr` remains unchanged and can be used in cases in
  which a compact binary representation is desired. For JSON state this does not
  save much data (e.g. the Bech32 address
  cosmos1pfq05em6sfkls66ut4m2257p7qwlk448h8mysz takes 45 bytes as direct ASCII
  and 28 bytes when its canonical representation is base64 encoded). For
  fixed-length database keys `CanonicalAddr` remains handy though.

- Replace `StakingMsg::Withdraw` with `DistributionMsg::SetWithdrawAddress` and
  `DistributionMsg::WithdrawDelegatorReward`. `StakingMsg::Withdraw` was a
  shorthand for the two distribution messages. However, it was unintuitive
  because it did not set the address for one withdrawal only but for all
  following withdrawals. Since withdrawals are [triggered by different
  events][distribution docs] such as validators changing their commission rate,
  an address that was set for a one-time withdrawal would be used for future
  withdrawals not considered by the contract author.

  If the contract never set a withdrawal address other than the contract itself
  (`env.contract.address`), you can simply replace `StakingMsg::Withdraw` with
  `DistributionMsg::WithdrawDelegatorReward`. It is then never changed from the
  default. Otherwise, you need to carefully track what the current withdrawal
  address is. A one-time change can be implemented by emitting 3 messages:

  1. `SetWithdrawAddress { address: recipient }` to temporarily change the
     recipient
  2. `WithdrawDelegatorReward { validator }` to do a manual withdrawal from the
     given validator
  3. `SetWithdrawAddress { address: env.contract.address.into() }` to change it
     back for all future withdrawals

  [distribution docs]:
    https://docs.cosmos.network/main/build/modules/distribution

- The block time in `env.block.time` is now a `Timestamp` which stores
  nanosecond precision. `env.block.time_nanos` was removed. If you need the
  components as before, use
  ```rust
  let seconds = env.block.time.nanos() / 1_000_000_000;
  let nsecs = env.block.time.nanos() % 1_000_000_000;
  ```

## 0.12 -> 0.13

- The minimum Rust supported version for 0.13 is 1.47.0. Verify your Rust
  version is >= 1.47.0 with: `rustc --version`

- Update CosmWasm dependencies in Cargo.toml (skip the ones you don't use):

  ```
  [dependencies]
  cosmwasm-std = "0.13.0"
  cosmwasm-storage = "0.13.0"
  # ...

  [dev-dependencies]
  cosmwasm-schema = "0.13.0"
  cosmwasm-vm = "0.13.0"
  # ...
  ```

## 0.11 -> 0.12

- Update CosmWasm dependencies in Cargo.toml (skip the ones you don't use):

  ```
  [dependencies]
  cosmwasm-std = "0.12.0"
  cosmwasm-storage = "0.12.0"
  # ...

  [dev-dependencies]
  cosmwasm-schema = "0.12.0"
  cosmwasm-vm = "0.12.0"
  # ...
  ```

- In your contract's `.cargo/config` remove `--features backtraces`, which is
  now available in Rust nightly only:

  ```diff
  @@ -1,6 +1,6 @@
   [alias]
   wasm = "build --release --target wasm32-unknown-unknown"
   wasm-debug = "build --target wasm32-unknown-unknown"
  -unit-test = "test --lib --features backtraces"
  +unit-test = "test --lib"
   integration-test = "test --test integration"
   schema = "run --example schema"
  ```

  In order to use backtraces for debugging, run
  `RUST_BACKTRACE=1 cargo +nightly unit-test --features backtraces`.

- Rename the type `Extern` to `Deps`, and radically simplify the
  `init`/`handle`/`migrate`/`query` entrypoints. Rather than
  `&mut Extern<S, A, Q>`, use `DepsMut`. And instead of `&Extern<S, A, Q>`, use
  `Deps`. If you ever pass eg. `foo<A: Api>(api: A)` around, you must now use
  dynamic trait pointers: `foo(api: &dyn Api)`. Here is the quick search-replace
  guide on how to fix `contract.rs`:

  _In production (non-test) code:_

  - `<S: Storage, A: Api, Q: Querier>` => ``
  - `&mut Extern<S, A, Q>` => `DepsMut`
  - `&Extern<S, A, Q>` => `Deps`
  - `&mut deps.storage` => `deps.storage` where passing into `state.rs` helpers
  - `&deps.storage` => `deps.storage` where passing into `state.rs` helpers

  On the top, remove `use cosmwasm_std::{Api, Extern, Querier, Storage}`. Add
  `use cosmwasm_std::{Deps, DepsMut}`.

  _In test code only:_

  - `&mut deps,` => `deps.as_mut(),`
  - `&deps,` => `deps.as_ref(),`

  You may have to add `use cosmwasm_std::{Storage}` if the compile complains
  about the trait

  _If you use cosmwasm-storage, in `state.rs`:_

  - `<S: Storage>` => ``
  - `<S: ReadonlyStorage>` => ``
  - `<S,` => `<`
  - `&mut S` => `&mut dyn Storage`
  - `&S` => `&dyn Storage`

- If you have any references to `ReadonlyStorage` left after the above, please
  replace them with `Storage`

## 0.10 -> 0.11

- Update CosmWasm dependencies in Cargo.toml (skip the ones you don't use):

  ```
  [dependencies]
  cosmwasm-std = "0.11.0"
  cosmwasm-storage = "0.11.0"
  # ...

  [dev-dependencies]
  cosmwasm-schema = "0.11.0"
  cosmwasm-vm = "0.11.0"
  # ...
  ```

- Contracts now support any custom error type `E: ToString + From<StdError>`.
  Previously this has been `StdError`, which you can still use. However, you can
  now create a much more structured error experience for your unit tests that
  handles exactly the error cases of your contract. In order to get a convenient
  implementation for `ToString` and `From<StdError>`, we use the crate
  [thiserror](https://crates.io/crates/thiserror), which needs to be added to
  the contracts dependencies in `Cargo.toml`. To create the custom error, create
  an error module `src/errors.rs` as follows:

  ```rust
  use cosmwasm_std::{CanonicalAddr, StdError};
  use thiserror::Error;

  // thiserror implements Display and ToString if you
  // set the `#[error("…")]` attribute for all cases
  #[derive(Error, Debug)]
  pub enum MyCustomError {
      #[error("{0}")]
      // let thiserror implement From<StdError> for you
      Std(#[from] StdError),
      // this is whatever we want
      #[error("Permission denied: the sender is not the current owner")]
      NotCurrentOwner {
          expected: CanonicalAddr,
          actual: CanonicalAddr,
      },
      #[error("Messages empty. Must reflect at least one message")]
      MessagesEmpty,
  }
  ```

  Then add `mod errors;` to `src/lib.rs` and `use crate::errors::MyCustomError;`
  to `src/contract.rs`. Now adapt the return types as follows:

  - `fn init`: `Result<InitResponse, MyCustomError>`,
  - `fn migrate` (if you have it): `Result<MigrateResponse, MyCustomError>`,
  - `fn handle`: `Result<HandleResponse, MyCustomError>`,
  - `fn query`: `Result<Binary, MyCustomError>`.

  If one of your functions does not use the custom error, you can continue to
  use `StdError` as before. I.e. you can have `handle` returning
  `Result<HandleResponse, MyCustomError>` and `query` returning
  `StdResult<Binary>`.

  You can have a top-hevel `init`/`migrate`/`handle`/`query` that returns a
  custom error but some of its implementations only return errors from the
  standard library (`StdResult<HandleResponse>` aka.
  `Result<HandleResponse, StdError>`). Then use `Ok(std_result?)` to convert
  between the result types. E.g.

  ```rust
  pub fn handle<S: Storage, A: Api, Q: Querier>(
      deps: &mut Extern<S, A, Q>,
      env: Env,
      msg: HandleMsg,
  ) -> Result<HandleResponse, StakingError> {
      match msg {
          // conversion to Result<HandleResponse, StakingError>
          HandleMsg::Bond {} => Ok(bond(deps, env)?),
          // this already returns Result<HandleResponse, StakingError>
          HandleMsg::_BondAllTokens {} => _bond_all_tokens(deps, env),
      }
  }
  ```

  or

  ```rust
  pub fn init<S: Storage, A: Api, Q: Querier>(
      deps: &mut Extern<S, A, Q>,
      env: Env,
      msg: InitMsg,
  ) -> Result<InitResponse, HackError> {
      // …

      let mut ctx = Context::new();
      ctx.add_attribute("Let the", "hacking begin");
      Ok(ctx.try_into()?)
  }
  ```

  Once you get familiar with the concept, you can create different error types
  for each of the contract's functions.

  You can also try a different error library than
  [thiserror](https://crates.io/crates/thiserror). The
  [staking development contract](https://github.com/CosmWasm/cosmwasm/tree/main/contracts/staking)
  shows how this would look like using [snafu](https://crates.io/crates/snafu).

- Change order of arguments such that `storage` is always first followed by
  namespace in `Bucket::new`, `Bucket::multilevel`, `ReadonlyBucket::new`,
  `ReadonlyBucket::multilevel`, `PrefixedStorage::new`,
  `PrefixedStorage::multilevel`, `ReadonlyPrefixedStorage::new`,
  `ReadonlyPrefixedStorage::multilevel`, `bucket`, `bucket_read`, `prefixed` and
  `prefixed_read`.

  ```rust
  // before
  let mut bucket = bucket::<_, Data>(b"data", &mut store);

  // after
  let mut bucket = bucket::<_, Data>(&mut store, b"data");
  ```

- Rename `InitResponse::log`, `MigrateResponse::log` and `HandleResponse::log`
  to `InitResponse::attributes`, `MigrateResponse::attributes` and
  `HandleResponse::attributes`. Replace calls to `log` with `attr`:

  ```rust
  // before
  Ok(HandleResponse {
    log: vec![log("action", "change_owner"), log("owner", owner)],
    ..HandleResponse::default()
  })

  // after
  Ok(HandleResponse {
    attributes: vec![attr("action", "change_owner"), attr("owner", owner)],
    ..HandleResponse::default()
  })
  ```

- Rename `Context::add_log` to `Context::add_attribute`:

  ```rust
  // before
  let mut ctx = Context::new();
  ctx.add_log("action", "release");
  ctx.add_log("destination", &to_addr);

  // after
  let mut ctx = Context::new();
  ctx.add_attribute("action", "release");
  ctx.add_attribute("destination", &to_addr);
  ```

- Add result type to `Bucket::update` and `Singleton::update`:

  ```rust
  // before
  bucket.update(b"maria", |mayd: Option<Data>| {
    let mut d = mayd.ok_or(StdError::not_found("Data"))?;
    old_age = d.age;
    d.age += 1;
    Ok(d)
  })

  // after
  bucket.update(b"maria", |mayd: Option<Data>| -> StdResult<_> {
    let mut d = mayd.ok_or(StdError::not_found("Data"))?;
    old_age = d.age;
    d.age += 1;
    Ok(d)
  })
  ```

- Remove all `canonical_length` arguments from mock APIs in tests:

  ```rust
  // before
  let mut deps = mock_dependencies(20, &[]);
  let mut deps = mock_dependencies(20, &coins(123456, "gold"));
  let deps = mock_dependencies_with_balances(20, &[(&rich_addr, &rich_balance)]);
  let api = MockApi::new(20);

  // after
  let mut deps = mock_dependencies(&[]);
  let mut deps = mock_dependencies(&coins(123456, "gold"));
  let deps = mock_dependencies_with_balances(&[(&rich_addr, &rich_balance)]);
  let api = MockApi::default();
  ```

- Add `MessageInfo` as separate arg after `Env` for `init`, `handle`, `migrate`.
  Add `Env` arg to `query`. Use `info.sender` instead of `env.message.sender`
  and `info.sent_funds` rather than `env.message.sent_funds`. Just changing the
  function signatures of the 3-4 export functions should be enough, then the
  compiler will warn you anywhere you use `env.message`

  ```rust
  // before
  pub fn init<S: Storage, A: Api, Q: Querier>(
      deps: &mut Extern<S, A, Q>,
      env: Env,
      msg: InitMsg,
  ) {
      deps.storage.set(
          CONFIG_KEY,
          &to_vec(&State {
              verifier: deps.api.canonical_address(&msg.verifier)?,
              beneficiary: deps.api.canonical_address(&msg.beneficiary)?,
              funder: deps.api.canonical_address(&env.message.sender)?,
          })?,
      );
  }

  // after
  pub fn init<S: Storage, A: Api, Q: Querier>(
      deps: &mut Extern<S, A, Q>,
      _env: Env,
      info: MessageInfo,
      msg: InitMsg,
  ) {
      deps.storage.set(
          CONFIG_KEY,
          &to_vec(&State {
              verifier: deps.api.canonical_address(&msg.verifier)?,
              beneficiary: deps.api.canonical_address(&msg.beneficiary)?,
              funder: deps.api.canonical_address(&info.sender)?,
          })?,
      );
  }
  ```

- Test code now has `mock_info` which takes the same args `mock_env` used to.
  You can just pass `mock_env()` directly into the function calls unless you
  need to change height/time.
- One more object to pass in for both unit and integration tests. To do this
  quickly, I just highlight all copies of `env` and replace them with `info`
  (using Ctrl+D in VSCode or Alt+J in IntelliJ). Then I select all `deps, info`
  sections and replace that with `deps, mock_env(), info`. This fixes up all
  `init` and `handle` calls, then just add an extra `mock_env()` to the query
  calls.

  ```rust
  // before: unit test
  let env = mock_env(creator.as_str(), &[]);
  let res = init(&mut deps, env, msg).unwrap();

  let query_response = query(&deps, QueryMsg::Verifier {}).unwrap();

  // after: unit test
  let info = mock_info(creator.as_str(), &[]);
  let res = init(&mut deps, mock_env(), info, msg).unwrap();

  let query_response = query(&deps, mock_env(), QueryMsg::Verifier {}).unwrap();

  // before: integration test
  let env = mock_env("creator", &coins(1000, "earth"));
  let res: InitResponse = init(&mut deps, env, msg).unwrap();

  let query_response = query(&mut deps, QueryMsg::Verifier {}).unwrap();

  // after: integration test
  let info = mock_info("creator", &coins(1000, "earth"));
  let res: InitResponse = init(&mut deps, mock_env(), info, msg).unwrap();

  let query_response = query(&mut deps, mock_env(), QueryMsg::Verifier {}).unwrap();
  ```

## 0.9 -> 0.10

- Update CosmWasm dependencies in Cargo.toml (skip the ones you don't use):

  ```
  [dependencies]
  cosmwasm-std = "0.10.0"
  cosmwasm-storage = "0.10.0"
  # ...

  [dev-dependencies]
  cosmwasm-schema = "0.10.0"
  cosmwasm-vm = "0.10.0"
  # ...
  ```

Integration tests:

- Calls to `Api::human_address` and `Api::canonical_address` now return a pair
  of result and gas information. Change

  ```rust
  // before
  verifier: deps.api.canonical_address(&verifier).unwrap(),

  // after
  verifier: deps.api.canonical_address(&verifier).0.unwrap(),
  ```

  The same applies for all calls of `Querier` and `Storage`.

All Tests:

All usages of `mock_env` will have to remove the first argument (no need of
API).

```rust
// before
let env = mock_env(&deps.api, "creator", &coins(1000, "earth"));

// after
let env = mock_env("creator", &coins(1000, "earth"));
```

Contracts:

- All code that uses `message.sender` or `contract.address` should deal with
  `HumanAddr` not `CanonicalAddr`. Many times this means you can remove a
  conversion step.

## 0.8 -> 0.9

- Update CosmWasm dependencies in Cargo.toml (skip the ones you don't use):

  ```
  [dependencies]
  cosmwasm-std = "0.9.0"
  cosmwasm-storage = "0.9.0"
  # ...

  [dev-dependencies]
  cosmwasm-schema = "0.9.0"
  cosmwasm-vm = "0.9.0"
  # ...
  ```

`lib.rs`:

- The C export boilerplate can now be reduced to the following code (see e.g. in
  [hackatom/src/lib.rs](https://github.com/CosmWasm/cosmwasm/blob/0a5b3e8121/contracts/hackatom/src/lib.rs)):

  ```rust
  mod contract; // contains init, handle, query
  // maybe additional modules here

  #[cfg(target_arch = "wasm32")]
  cosmwasm_std::create_entry_points!(contract);
  ```

Contract code and uni tests:

- `cosmwasm_storage::get_with_prefix`, `cosmwasm_storage::set_with_prefix`,
  `cosmwasm_storage::RepLog::commit`, `cosmwasm_std::ReadonlyStorage::get`,
  `cosmwasm_std::ReadonlyStorage::range`, `cosmwasm_std::Storage::set` and
  `cosmwasm_std::Storage::remove` now returns the value directly that was
  wrapped in a result before.
- Error creator functions are now in type itself, e.g.
  `StdError::invalid_base64` instead of `invalid_base64`. The free functions are
  deprecated and will be removed before 1.0.
- Remove `InitResponse.data` in `init`. Before 0.9 this was not stored to chain
  but ignored.
- Use `cosmwasm_storage::transactional` instead of the removed
  `cosmwasm_storage::transactional_deps`.
- Replace `cosmwasm_std::Never` with `cosmwasm_std::Empty`.

Integration tests:

- Replace `cosmwasm_vm::ReadonlyStorage` with `cosmwasm_vm::Storage`, which now
  contains all backend storage methods.
- Storage getters (and iterators) now return a result of
  `(Option<Vec<u8>>, u64)`, where the first component is the element and the
  second one is the gas cost. Thus in a few places `.0` must be added to access
  the element.

## 0.7.2 -> 0.8

### Update wasm code

`Cargo.toml` dependencies:

- Update to `schemars = "0.7"`
- Replace `cosmwasm = "0.7"` with `cosmwasm-std = "0.8"`
- Replace `cosmwasm-vm = "0.7"` with `cosmwasm-vm = "0.8"`
- Replace `cw-storage = "0.2"` with `cosmwasm-storage = "0.8"`
- Remove explicit `snafu` dependency. `cosmwasm_std` still uses it internally
  but doesn't expose snafu specifics anymore. See more details on errors below.

(Note: until release of `0.8`, you need to use git references for all
`cosmwasm_*` packages)

`Cargo.toml` features:

- Replace `"cosmwasm/backtraces"` with `"cosmwasm-std/backtraces"`

Imports:

- Replace all `use cosmwasm::X::Y` with `use cosmwasm_std::Y`, except for mock
- Replace all `use cosmwasm::mock::Y` with `use cosmwasm_std::testing::Y`. This
  should only be used in test code.
- Replace `cw_storage:X` with `cosmwasm_storage::X`
- Replace `cosmwasm_std::Response` with `cosmwasm_std::HandleResponse` and
  `cosmwasm_std::InitResponse` (different type for each call)

`src/lib.rs`:

This has been re-written, but is generic boilerplate and should be (almost) the
same in all contracts:

- copy the new version from
  [`contracts/queue`](https://github.com/CosmWasm/cosmwasm/blob/main/contracts/queue/src/lib.rs)
- Add `pub mod XYZ` directives for any modules you use besides `contract`

Contract Code:

- Add query to extern:
  - Before: `my_func<S: Storage, A: Api>(deps: &Extern<S, A>, ...`
  - After: `my_func<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, ...`
  - Remember to add `use cosmwasm_std::Querier;`
- `query` now returns `StdResult<Binary>` instead of `Result<Vec<u8>>`
  - You can also replace `to_vec(...)` with `to_binary(...)`
- No `.context(...)` is required after `from_slice` and `to_vec`, they return
  proper `cosmwasm_std::Error` variants on errors.
- `env.message.signer` becomes `env.message.sender`.
- If you used `env.contract.balance`, you must now use the querier. The
  following code block should work:

  ```rust
  // before (in env)
  let foo = env.contract.balance;

  // after (query my balance)
  let contract_addr = deps.api.human_address(&env.contract.address)?;
  let balance = deps.querier.query_all_balances(&contract_addr)?;
  let foo = balance.amount;
  ```

- Update the `CosmosMsg` enums used:

  - `CosmosMsg::Send{}` => `CosmosMsg::Bank(BankMsg::Send{})`
  - `CosmosMsg::Opaque{ data }` => `CosmosMsg::Native{ msg }`
  - `CosmosMsg::Contract` => `CosmosMsg::Wasm(WasmMsg::Execute{})`

- Complete overhaul of `cosmwasm::Error` into `cosmwasm_std::StdError`:
  - Auto generated snafu error constructor structs like `NotFound`/`ParseErr`/…
    have been privatized in favor of error generation helpers like
    `not_found`/`parse_err`/…
  - All error generator functions now return errors instead of results, such
    that e.g. `return unauthorized();` becomes `return Err(unauthorized());`
  - Error cases don't contain `source` fields anymore. Instead source errors are
    converted to standard types like `String`. For this reason, both
    `snafu::ResultExt` and `snafu::OptionExt` cannot be used anymore. An error
    wrapper now looks like `.map_err(invalid_base64)` and an `Option::None` to
    error mapping looks like `.ok_or_else(|| not_found("State"))`.
  - Backtraces became optional. Use `RUST_BACKTRACE=1` to enable them for unit
    tests.
  - `Utf8Err`/`Utf8StringErr` merged into `StdError::InvalidUtf8`
  - `Base64Err` renamed into `StdError::InvalidBase64`
  - `ContractErr`/`DynContractErr` merged into `StdError::GenericErr`, thus both
    `contract_err` and `dyn_contract_err` must be replaced with `generic_err`.
  - The unused `ValidationErr` was removed

At this point `cargo wasm` should pass.

### Update test code

Both:

- Update all imports from `cosmwasm::mock::*` to `cosmwasm_std::testing::*`
- Use `from_binary` not `from_slice` on all query responses (update imports)
  - `from_slice(res.as_slice())` -> `from_binary(&res)`
- Replace `coin("123", "FOO")` with `coins(123, "FOO")`. We renamed it to coins
  to be more explicit that it returns `Vec<Coin>`, and now accept a `u128` as
  the first argument for better type-safety. `coin` is now an alias to
  `Coin::new` and returns one `Coin`.
- Remove the 4th argument (contract balance) from all calls to `mock_env`, this
  is no longer stored in the environment.
- `dependencies` was renamed to `mock_dependencies`. `mock_dependencies` and
  `mock_instance` take a 2nd argument to set the contract balance (visible for
  the querier). If you need to set more balances, use `mock_XX_with_balances`.
  The following code block explains:

  ```rust
  // before: balance as last arg in mock_env
  let mut deps = dependencies(20);
  let env = mock_env(&deps.api, "creator", &coins(15, "earth"), &coins(1015, "earth"));

  // after: balance as last arg in mock_dependencies
  let mut deps = mock_dependencies(20, &coins(1015, "earth"));
  let env = mock_env(&deps.api, "creator", &coins(15, "earth"));
  ```

Unit Tests:

- Replace `dependencies` with `mock_dependencies`

Integration Tests:

- We no longer check errors as strings but have rich types:
  - Before:
    `match err { ContractResult::Err(msg) => assert_eq!(msg, "Unauthorized"), ... }`
  - After: `match err { Err(StdError::Unauthorized{ .. }) => {}, ... }`
- Remove all imports/use of `ContractResult`
- You must specify `CosmosMsg::Native` type when calling
  `cosmwasm_vm::testing::{handle, init}`. You will want to
  `use cosmwasm_std::{HandleResult, InitResult}` or
  `use cosmwasm_std::{HandleResponse, InitResponse}`. If you don't use custom
  native types, simply update calls as follows:
  - `let res = init(...)` => `let res: InitResult = init(...)`
  - `let res = init(...).unwrap()` =>
    `let res: InitResponse = init(...).unwrap()`
  - `let res = handle(...)` => `let res: HandleResult = handle(...)`
  - `let res = handle(...).unwrap()` =>
    `let res: HandleResponse = handle(...).unwrap()`

### Update schema code

All helper functions have been moved into a new `cosmwasm-schema` package.

- Add `cosmwasm-schema = "0.8"` to `[dev-dependencies]` in `Cargo.toml`
- Remove `serde_json` `[dev-dependency]` if there, as cosmwasm-schema will
  handle JSON output internally.
- Update `examples/schema.rs` to look
  [more like queue](https://github.com/CosmWasm/cosmwasm/blob/v0.8.0/contracts/queue/examples/schema.rs),
  but replacing all the imports and type names with those you currently have.
- Regenerate schemas with `cargo schema`

### Polishing

After so many changes, remember to let the linters do their jobs.

- `cargo fmt`
- `cargo clippy`
