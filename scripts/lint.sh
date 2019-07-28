#!/usr/bin/env bash

set -euo pipefail

yarn install
PATH="$(yarn bin):$PATH"
export PATH
cd "$(pkg-dir)"

set -x

# Yarn orchestration

## Lint package.json
pjv

# Rust sources

## Format with rustfmt
cargo fmt
## Lint with Clippy
cargo clippy --all-targets --all-features
## Lint docs
cargo doc --no-deps --all

# Lint Ruby sources

lint_ruby_sources() {
  pushd "$@" >/dev/null
  bundle install >/dev/null
  bundle exec rubocop -a
  popd >/dev/null
}

## Backend Core and StdLib
lint_ruby_sources artichoke-backend/src/extn
## Frontend bins
lint_ruby_sources artichoke-frontend/ruby
## spec-runner
lint_ruby_sources spec-runner/src

# C sources

## Format with clang-format
./scripts/format-c.sh --format

# Shell sources

## Format with shfmt
shfmt -f . | grep -v target/ | grep -v node_modules/ | grep -v spec-runner/spec/ | grep -v vendor/ | xargs shfmt -i 2 -ci -s -w
## Lint with shellcheck
shfmt -f . | grep -v target/ | grep -v node_modules/ | grep -v spec-runner/spec/ | grep -v vendor/ | xargs shellcheck

# Text sources (e.g. HTML, Markdown)

## Format with prettier
yarn run prettier --write --prose-wrap always \
  './*.{css,html,js,json,md}' \
  '{!(target),!(node_modules)}**/*.{css,html,js,json,md}'