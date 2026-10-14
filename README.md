# Stellar `upto` settlement kernel

This workspace contains the experimental Soroban contract for Stellar's x402
`upto` payment flow. It is intentionally small and stateless: the buyer signs a
maximum charge, the bound facilitator signs the final charge, and the contract
moves a SEP-41 token directly from the buyer to the recipient.

The ABI is not stable and the contract has not been audited. The source now
uses explicit `valid_after_ledger` and `deadline_ledger` bounds. The older
single-deadline testnet deployment is incompatible with this source and is
recorded in `deployments/stellar-testnet.json` under `history`. The current ABI is
deployed and its settlement transactions are recorded under `current`.
The contract is an experimental component, not an audited production release.
Its entry points and known limitations are summarized below.

## Entry points

- `settle` spends an existing allowance from the buyer to the kernel.
- `settle_with_approval` creates a finite allowance and spends it atomically.

Both paths reject non-positive charges, charges above the buyer's maximum,
inverted validity windows, calls before the signed lower bound, and calls after
the signed deadline. The approval path derives both allowance amount and expiry
from the signed settlement terms.

The contract has no owner, upgrade, pause, withdrawal, arbitrary-call, or
storage interface. Canonical Kernel events are intentionally deferred until
their upstream schema is agreed; the token's transfer and approval events are
still emitted normally.

Three properties are weaker than the entry-point names suggest:

- `max_amount` caps one authorization, not one set of terms. The two entry
  points are separate preimages, so a buyer who signs both has authorized two
  settlements.
- `settle_with_approval` overwrites any allowance the buyer already granted the
  kernel, because SEP-41 `approve` sets rather than adds.
- A zero-value settlement produces no transaction and so consumes no host
  nonce. At zero, single use is the facilitator's journal, not the chain.

## Development

From this directory:

```sh
cargo test --offline --workspace
cargo fmt --all --check
cargo clippy --offline --workspace --all-targets -- -D warnings
stellar contract build --package upto-settlement-kernel
```

The checked-in lockfile and Rust toolchain pin make local and CI builds use the
same dependency graph.
