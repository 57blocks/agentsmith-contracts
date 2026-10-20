# Stellar `upto` settlement kernel

This workspace contains the experimental Soroban contract for Stellar's x402
`upto` payment flow. It is intentionally small and stateless: the buyer signs a
maximum charge, the bound facilitator signs the final charge, and the contract
moves a SEP-41 token directly from the buyer to the recipient.

The ABI is not stable and the contract has not been audited. The source uses
explicit `valid_after_ledger` and `deadline_ledger` bounds and a single `settle`
entry point. Deployment records in `deployments/stellar-testnet.json` describe
previous deployments, not evidence that this ABI change has been deployed.
The contract is an experimental component, not an audited production release.
Its entry points and known limitations are summarized below.

## Entry point

`settle(terms, actual_amount)` creates a finite allowance for `terms.max_amount`
until `terms.deadline_ledger`, then transfers `actual_amount` directly from the
buyer to `terms.pay_to`. Approval and transfer execute atomically; a failed
transfer rolls back the approval.

The buyer authorizes `terms` and the nested token `approve` invocation. The bound
facilitator authorizes `terms` and `actual_amount`. Existing allowance does not
remove the requirement for the buyer to authorize the nested approval.

Settlement rejects non-positive charges, charges above the buyer's maximum,
inverted validity windows, calls before the signed lower bound, and calls after
the signed deadline. It derives both allowance amount and expiry
from the signed settlement terms.

This replaces both the old allowance-only `settle` and `settle_with_approval`.
Callers must use the new `settle` invocation and authorization tree. Old
`settle_with_approval` signatures bind a different function name; old `settle`
signatures lack the required nested approval. Neither can be reused as-is.
Deploy the new contract and update buyer and facilitator configuration together.

The contract has no owner, upgrade, pause, withdrawal, arbitrary-call, or
storage interface. Canonical Kernel events are intentionally deferred until
their upstream schema is agreed; the token's transfer and approval events are
still emitted normally.

Known limitations:

- `max_amount` caps one authorization, not all authorizations with the same
  terms. Separately signed authorizations can each settle up to that maximum.
- `settle` overwrites any allowance the buyer already granted the
  kernel, because SEP-41 `approve` sets rather than adds.
- Remaining allowance is not cleared after settlement; it expires at the signed
  deadline.
- The contract rejects zero-value settlements. A facilitator that handles zero
  without an on-chain transaction must enforce single use in its own journal.

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
