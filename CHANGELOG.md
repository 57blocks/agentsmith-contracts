# Changelog

## upto-settlement-kernel

### two-boundary-v2 — current

Deployed on testnet (`CBB7WCIQ…XRE3`) and pubnet (`CCV4RALF…ODNI`) from source commit
`dccd19c`; WASM hash `ba832b45…e916`.

- One entry point, `settle(terms, actual_amount)`. It approves the kernel for
  `terms.max_amount` until `terms.deadline_ledger` and transfers `actual_amount` from the payer to
  `terms.pay_to` in the same invocation.
- Replaces both the earlier allowance-only `settle` and `settle_with_approval`. Signatures made for
  those functions cannot be reused: `settle_with_approval` binds a different function name, and the
  old `settle` authorization lacks the nested token `approve`.
- The facilitator authorizes the full arguments `(terms, actual_amount)`.

### Earlier ABIs

`two-boundary-v1`, `single-deadline-v0` and `valid-after-plus-approval-terms-v0` are deprecated.
Their contract IDs are kept in `deployments/stellar-testnet.json` under `history`. Do not use them.
