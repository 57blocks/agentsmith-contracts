# Security policy

These contracts move user funds. They have **not been audited**.

## Reporting a vulnerability

Email **agentsmith@57blocks.com**. Do not open a public issue, pull request or discussion for a
suspected vulnerability.

Please include:

- the contract and network (and contract ID, if it concerns a deployment);
- the source commit you looked at;
- a description of the issue and how to reproduce it, ideally as a failing test.

We will acknowledge your report, keep you informed while we investigate, and credit you when the
fix is published unless you ask us not to.

## Scope

- `upto-settlement-kernel` and its deployments listed in `deployments/`.
- `e2e-simple-account` is a test fixture that must never hold funds; reports about it are welcome
  but are not treated as production issues.

The contracts have no owner, admin or upgrade function. A fixed version is a new deployment at a
new contract ID; the AgentSmith facilitator and SDK then move to it.
