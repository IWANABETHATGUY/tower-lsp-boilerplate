set windows-shell := ["powershell"]


build:
  build

lint:
  cargo clippy --workspace --all-targets -- --deny warnings

fmt:
  cargo fmt --all
  pnpm fmt
