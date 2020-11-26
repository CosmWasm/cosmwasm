#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

# Temporary incomplete testing command for development
(cd packages/vm \
  && cargo check --tests \
  && cargo check --features iterator --tests \
  && cargo test --features iterator checksum:: \
  && cargo test --features iterator conversion:: \
  && cargo test --features iterator compatibility \
  && cargo test --features iterator errors:: \
  && cargo test --features iterator features:: \
  && cargo test --features iterator ffi:: \
  && cargo test --features iterator limiting_tunables:: \
  && cargo test --features iterator memory:: \
  && cargo test --features iterator modules:: \
  && cargo test --features iterator testing:: \
  && cargo test --features iterator init_cached_contract \
  && cargo test --features iterator load_wasm_errors_for_non_existent_id \
  && cargo test --features iterator load_wasm_from_disk_works \
  && cargo test --features iterator load_wasm_from_disk_works_in_subfolder \
  && cargo test --features iterator get_instance_finds_cached_module \
  && cargo test --features iterator load_wasm_errors_for_corrupted_wasm \
  && cargo test --features iterator get_instance_finds_cached_instance \
  && cargo test --features iterator load_wasm_works \
  && cargo test --features iterator save_wasm_rejects_invalid_contract \
  && cargo test --features iterator save_wasm_to_disk_fails_on_non_existent_dir \
  && cargo test --features iterator save_wasm_to_disk_works_for_same_data_multiple_times \
  && cargo test --features iterator load_wasm_works_across_multiple_cache_instances \
  && cargo test --features iterator save_wasm_works \
  && cargo test --features iterator run_cached_contract \
  && cargo test --features iterator use_multiple_cached_instances_of_same_contract \
  && cargo test --features iterator save_wasm_allows_saving_multiple_times \
  && cargo test --features iterator call_func_works \
  && cargo test --features iterator required_features_works_for_many_exports \
  && cargo test --features iterator get_memory_size_works \
  && cargo test --features iterator required_features_works \
  && cargo test --features iterator read_memory_errors_when_when_length_is_too_long \
  && cargo test --features iterator with_storage_safe_for_panic \
  && cargo test --features iterator with_querier_works_readonly \
  && cargo test --features iterator with_querier_allows_updating_balances \
  && cargo test --features iterator set_storage_readonly_works \
  && cargo test --features iterator with_storage_works \
  && cargo test --features iterator write_and_read_memory_works \
  && cargo clippy --features iterator -- -D warnings)

# Contracts
for contract_dir in contracts/*/; do
  (cd "$contract_dir" && cargo wasm && cargo integration-test) || break;
done
