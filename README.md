# AgentSmith Soroban contracts

Soroban contracts for the Stellar [x402](https://github.com/x402-foundation/x402) `upto` payment
scheme, used by the [AgentSmith](https://github.com/57blocks/agentsmith) facilitator and SDK.

> [!WARNING]
> **These contracts have not been audited.** They move user funds. The ABI is experimental and
> may change in a new deployment. Review the code yourself before relying on it, and report
> vulnerabilities as described in [SECURITY.md](SECURITY.md).

## What the kernel does

In x402 `upto`, a buyer authorizes a **maximum** charge, the service runs, and the facilitator then
settles the **actual** amount, which may be lower. `upto-settlement-kernel` enforces that on chain:

- the buyer signs the settlement terms once, including the maximum;
- the facilitator named in those terms signs the actual amount;
- the contract checks both and moves the actual amount of a SEP-41 token directly from the buyer to
  the recipient, in one atomic invocation.

The contract is stateless. It has no owner, admin, upgrade, pause, withdrawal, arbitrary-call or
storage interface, and it never holds funds. A new version is a new deployment at a new contract
ID; see [CHANGELOG.md](CHANGELOG.md).

## Deployments

Both networks run the same WASM, built from this repository at source commit `dccd19c`.

| Network            | Contract ID                                                | WASM SHA-256                                                       |
| ------------------ | ---------------------------------------------------------- | ------------------------------------------------------------------ |
| `stellar:pubnet`   | `CCV4RALFBEE2QLKPK6H5ZX24CYMKIFN3723J5J4TMDEFWAFLELGYODNI` | `ba832b45c0ac604f3aabca40203099556235da24c45e76fdb3bb56dc239de916` |
| `stellar:testnet`  | `CBB7WCIQOBOJ4G7JUPVOA3RI553HBSYQ3GLBJ25X7W7S7MRJIGVCXRE3` | `ba832b45c0ac604f3aabca40203099556235da24c45e76fdb3bb56dc239de916` |

Deployment details, verification evidence and deprecated contract IDs are in
[`deployments/`](deployments/). Use only the contract IDs above.

## Interface

### `settle(terms: SettlementTerms, actual_amount: i128)`

Validates the terms, checks both authorizations, then:

1. calls `approve(payer, kernel, max_amount, deadline_ledger)` on `asset`;
2. calls `transfer_from(kernel, payer, pay_to, actual_amount)` on `asset`.

If anything fails, the whole invocation reverts, including the approval. Nothing is returned;
the token emits its usual `approve` and `transfer` events. The kernel defines no events of its own
yet.

### `SettlementTerms`

| Field                | Type      | Meaning                                                                      |
| -------------------- | --------- | ---------------------------------------------------------------------------- |
| `payer`              | `Address` | The buyer whose tokens are spent.                                            |
| `asset`              | `Address` | The SEP-41 token contract.                                                   |
| `pay_to`             | `Address` | The recipient.                                                               |
| `facilitator`        | `Address` | The only address that may choose the actual amount.                          |
| `max_amount`         | `i128`    | The most that can be settled, in the token's atomic units. Must be positive. |
| `valid_after_ledger` | `u32`     | Settlement is refused before this ledger.                                    |
| `deadline_ledger`    | `u32`     | Settlement is refused after this ledger; the approval also expires here.     |

### Authorization

The transaction must carry two authorization entries:

```text
payer:        settle(terms)                         ← signs the terms, not the actual amount
              └─ asset.approve(payer, kernel, max_amount, deadline_ledger)
facilitator:  settle(terms, actual_amount)          ← signs the exact amount
```

The payer's entry must include the nested `approve`. An allowance the payer granted earlier does
not replace it.

### Errors

| Code | Name                    | When                                         |
| ---- | ----------------------- | -------------------------------------------- |
| 1    | `InvalidMaximum`        | `max_amount` is zero or negative.            |
| 2    | `InvalidActualAmount`   | `actual_amount` is zero or negative.         |
| 3    | `ExceedsMaximum`        | `actual_amount` is greater than `max_amount`. |
| 4    | `Expired`               | The current ledger is after `deadline_ledger`. |
| 5    | `NotYetValid`           | The current ledger is before `valid_after_ledger`. |
| 6    | `InvalidValidityWindow` | `valid_after_ledger` is after `deadline_ledger`. |

A missing or wrong authorization fails with the host's authorization error.

## Known limitations

- `max_amount` caps one authorization, not all authorizations with the same terms. Separately
  signed authorizations can each settle up to that maximum.
- `settle` overwrites any allowance the buyer already granted the kernel, because SEP-41 `approve`
  sets rather than adds.
- Remaining allowance is not cleared after settlement; it expires at `deadline_ledger`.
- Zero-value settlements are rejected. A facilitator that closes a zero charge without an on-chain
  transaction must enforce single use itself.

## Repository layout

| Path                      | Contents                                                                                   |
| ------------------------- | ------------------------------------------------------------------------------------------ |
| `upto-settlement-kernel/` | The kernel and its tests.                                                                  |
| `e2e-simple-account/`     | A minimal contract account used only by integration tests. Never deploy it to hold funds. |
| `deployments/`            | One record per network: contract IDs, WASM hashes, source commits and verification.        |

## Development

Requirements: the Rust toolchain pinned in `rust-toolchain.toml` (installed by `rustup`
automatically) and the [Stellar CLI](https://developers.stellar.org/docs/tools/cli) 23 or later.

```sh
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
stellar contract build --package upto-settlement-kernel
```

The build writes `target/wasm32v1-none/release/upto_settlement_kernel.wasm`. The checked-in
`Cargo.lock` and toolchain pin keep builds reproducible.

### Verify a deployment

Build at the source commit of the deployment, fetch the deployed WASM and compare the hashes:

```sh
git checkout dccd19c
stellar contract build --package upto-settlement-kernel
shasum -a 256 target/wasm32v1-none/release/upto_settlement_kernel.wasm

# pubnet: the CLI has no default mainnet RPC, so pass one
stellar contract fetch \
  --rpc-url "$MAINNET_RPC_URL" \
  --network-passphrase "Public Global Stellar Network ; September 2015" \
  --id CCV4RALFBEE2QLKPK6H5ZX24CYMKIFN3723J5J4TMDEFWAFLELGYODNI --out-file deployed.wasm
shasum -a 256 deployed.wasm

# testnet
stellar contract fetch --network testnet \
  --id CBB7WCIQOBOJ4G7JUPVOA3RI553HBSYQ3GLBJ25X7W7S7MRJIGVCXRE3 --out-file deployed-testnet.wasm
shasum -a 256 deployed-testnet.wasm
```

`MAINNET_RPC_URL` is any Stellar mainnet RPC endpoint; see the
[RPC providers](https://developers.stellar.org/docs/data/apis/rpc/providers) list. Every command
should print `ba832b45c0ac604f3aabca40203099556235da24c45e76fdb3bb56dc239de916`.

## Security

Not audited. See [SECURITY.md](SECURITY.md) to report a vulnerability.

## License

Licensed under the [Apache License, Version 2.0](LICENSE). Copyright 2026 57Blocks; see
[NOTICE](NOTICE).
