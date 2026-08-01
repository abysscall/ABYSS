
---

## ADR Immutability Rule

Once an ADR's status is `Accepted`, its architectural decision does not
change in place. If a decision needs to change:

1. Write a **new** ADR with the next available number.
2. In its header, declare `Supersedes: ADR-00XX` (full replacement) or
   `Amends: ADR-00XX` (partial refinement, original mostly still stands).
3. Update the superseded/amended ADR's own header to note
   `Status: Superseded by ADR-00YY` or `Status: Amended by ADR-00YY`,
   but leave its original content untouched.

ADR numbers are never reused, even if a document is later abandoned
before merge. The history of *why* a decision changed is as valuable as
the decision itself.
