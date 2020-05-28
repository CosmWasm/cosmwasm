#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

function print_usage() {
  echo "Usage: $0 NEW_VERSION"
  echo ""
  echo "e.g. $0 0.8.0"
}

if [ "$#" -ne 1 ]; then
    print_usage
    exit 1
fi

# Check repo
SCRIPT_DIR="$(realpath "$(dirname "$0")")"
if [[ "$(realpath "$SCRIPT_DIR/..")" != "$(pwd)" ]]; then
  echo "Script must be called from the repo root"
  exit 2
fi

# Ensure repo is not dirty
CHANGES_IN_REPO=$(git status --porcelain)
if [[ -n "$CHANGES_IN_REPO" ]]; then
    echo "Repository is dirty. Showing 'git status' and 'git --no-pager diff' for debugging now:"
    git status && git --no-pager diff
    exit 3
fi

NEW="$1"
OLD=$(sed -n -e 's/^version[[:space:]]*=[[:space:]]*"\(.*\)"/\1/p' packages/std/Cargo.toml)
echo "Updating old version $OLD to new version $NEW ..."

FILES_MODIFIED=()

for package_dir in packages/*/; do
  CARGO_TOML="$package_dir/Cargo.toml"
  sed -i '' -e "s/version[[:space:]]*=[[:space:]]*\"$OLD\"/version = \"$NEW\"/" "$CARGO_TOML"
  FILES_MODIFIED+=("$CARGO_TOML")
done

cargo +nightly build
FILES_MODIFIED+=("Cargo.lock")

for contract_dir in contracts/*/; do
  CARGO_TOML="$contract_dir/Cargo.toml"
  CARGO_LOCK="$contract_dir/Cargo.lock"

  sed -i '' -e "s/version[[:space:]]*=[[:space:]]*\"$OLD\"/version = \"$NEW\"/" "$CARGO_TOML"
  (cd "$contract_dir" && cargo build)

  FILES_MODIFIED+=("$CARGO_TOML" "$CARGO_LOCK")
done

echo "Staging ${FILES_MODIFIED[*]} ..."
git add "${FILES_MODIFIED[@]}"
git commit -m "Set version: $NEW"
