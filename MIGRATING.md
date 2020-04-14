# Migrating Contracts

This guide explains what is needed to upgrade contracts when migrating over
major releases of `cosmwasm`. Note that you can also view the
[complete CHANGELOG](./CHANGELOG.md) to understand the differences.

## 0.7.2 -> 0.8

### Update wasm code

`Cargo.toml` dependencies:

- Update to `schemars = "0.7"`
- Update to `snafu = "0.6.3"`
- Replace `cosmwasm = "0.7"` with `cosmwasm_std = "0.8"`
- Replace `cosmwasm_vm = "0.7"` with `cosmwasm_vm = "0.8"`
- Replace `cw_storage = "0.2"` with `cosmwasm_storage = "0.8"`

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
- `query` now returns `Result<Binary>` not `Result<Vec<u8>>`
  - You can also replace `to_vec(...)` with `to_binary(...)`
- No `.context(...)` is required after `from_slice` and `to_vec`, they return
  proper `cosmwasm_std::Error` variants on errors.
- If you need to access `env.contract.balance`, you must now use the querier.
  The following code block should work: **TODO**

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
- If you care about contract balance, you need to modify `mock_dependencies` or
  `mock_instance` to `mock_XX_with_balances`, where the extra argument contains
  the contract balance. The follow code block explains:
  ```rust
  // before: balance as last arg in mock_env
  let mut deps = mock_dependencies(20);
  let env = mock_env(&deps.api, creator, &coins(15, "earth"), &coins(1015, "earth"));

  // after: store balance in dependencies ("cosmos2contract" is the hardcoded contract address for all tests)
  let mut deps = mock_dependencies_with_balances(20, &[(&HumanAddr::from("cosmos2contract"), &coins(1015, "earth"))]);
  let env = mock_env(&deps.api, creator, &coins(15, "earth"));
  ```

Unit Tests:

- Replace `dependencies` with `mock_dependencies`

Integration Tests:

- We no longer check errors as strings but have rich types:
  - Before:
    `match err { ContractResult::Err(msg) => assert_eq!(msg, "Unauthorized"), ... }`
  - After: `match err { Err(ApiError::Unauthorized{ ..}) => {}, ... }`
- Remove all imports / use of `ContractResult`

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
