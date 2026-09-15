# Contract integration (`register_callback`)

`src/contracts/mod.rs` defines a `ContractClient` trait and a
`SorobanContractClient` HTTP implementation that calls
`pulsar-core-contracts`' `register_callback()` as the trusted `relay_signer`.

**The request/response shape (`RegisterCallbackWireRequest`/`Response`) is a
placeholder.** `pulsar-core-contracts` is a sibling repo (see the top of
`CLAUDE.md.pulsar-core`) that isn't cloned into this checkout — its
`EVENTS.md` and the contract's actual ABI have not been read. Per this repo's
standing brief: *"The contract's `register_callback()` signature and event
schema belong to `pulsar-core-contracts` — if you need a change there, open
the sibling repo, propose it as an ADR-style note, don't just assume a shape
and build against it."*

## Before this goes anywhere near a real deployment