# Migrating Contracts

This guide explains what is needed to upgrade contracts when migrating over
major releases of `cosmwasm`. Note that you can also view the
[complete CHANGELOG](./CHANGELOG.md) to understand the differences.

## 0.11 -> 0.10

- Contracts now support any custom error type `E: ToString + From<StdError>`.
  Previously this has been `StdError`, which you can still use. However, you can
  now create a much more structured error experience for your unit tests that
  handels exactly the error cases of your contract. In order to get a convenient
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

  If one of your funtions does not use the custom error, you can continue to use
  `StdError` as before. I.e. you can have `handle` returning
  `Result<HandleResponse, MyCustomError>` and `query` returning
  `StdResult<Binary>`.

  Once you got familiar with the concept, you can create different error types
  for each of the contract's functions.

  You can also try a different error library than
  [thiserror](https://crates.io/crates/thiserror). The
  [staking development contract](https://github.com/CosmWasm/cosmwasm/tree/master/contracts/staking)
  shows how this would look like using [snafu](https://crates.io/crates/snafu).

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

## 0.9 -> 0.10

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

`dependencies`/`dev-dependencies` in `Cargo.toml`:

- Replace `cosmwasm-schema = "0.8"` with `cosmwasm-schema = "0.9"`
- Replace `cosmwasm-std = "0.8"` with `cosmwasm-std = "0.9"`
- Replace `cosmwasm-storage = "0.8"` with `cosmwasm-storage = "0.9"`
- Replace `cosmwasm-vm = "0.8"` with `cosmwasm-vm = "0.9"`

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
  `cosmwasm_std::Storage::remove` now return the value directly that was wrapped
  in a result before.
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
  [`contracts/queue`](https://github.com/CosmWasm/cosmwasm/blob/master/contracts/queue/src/lib.rs)
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
    have been privatized in favour of error generation helpers like
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
  The follow code block explains:

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
- Remove all imports / use of `ContractResult`
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
  [more like queue](https://github.com/CosmWasm/cosmwasm/blob/master/contracts/queue/examples/schema.rs),
  but replacing all the imports and type names with those you currently have.
- Regenerate schemas with `cargo schema`

### Polishing

After so many changes, remember to let the linters do their jobs.

- `cargo fmt`
- `cargo clippy`
