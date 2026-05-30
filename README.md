# srvcs-or

The boolean OR primitive of the srvcs.cloud distributed standard library.

Its single concern: **the boolean OR of two operands.** It is a *leaf* — it
depends on no other service, computing `a || b` directly. Both operands must be
JSON booleans; this lets logic services compose it without ambiguity.

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/` | Service identity, concern, and dependency list |
| `POST` | `/` | Compute `a \|\| b` for two boolean operands |
| `GET` | `/healthz` `/readyz` `/metrics` `/openapi.json` | srvcs service standard surface |

```sh
curl -s -X POST localhost:8080/ -H 'content-type: application/json' -d '{"a": true, "b": false}'
# {"a":true,"b":false,"result":true}

curl -s -X POST localhost:8080/ -H 'content-type: application/json' -d '{"a": false, "b": false}'
# {"a":false,"b":false,"result":false}
```

`POST /` requires both `a` and `b` to be JSON booleans and returns their boolean
OR. Any non-boolean operand (number, string, `null`, array, or object) is a
client error and returns `422` with an `{"error": "..."}` body.

## Dependencies

None. `srvcs-or` is a leaf.

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `SRVCS_BIND_ADDR` | `0.0.0.0:8080` | Bind address |
| `SRVCS_ENV` | `development` | Environment label for logs |
| `RUST_LOG` | `info,tower_http=info` | Tracing filter |

## Local checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The full Nix gates and OCI image build are documented in
[`srvcs/platform`](https://github.com/srvcs/platform); CI runs them through the
shared `build-service.yml` workflow.

> Note: the `cargoHash` in `flake.nix` is inherited from the template and must be
> refreshed with a `nix build` before the Nix gates pass.
