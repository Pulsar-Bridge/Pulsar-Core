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

1. Clone `pulsar-core-contracts` as `../pulsar-core-contracts` (fill in its
   clone URL in `CLAUDE.md.pulsar-core` first).
2. Read `EVENTS.md` and the contract's `register_callback()` signature there.
3. Update `RegisterCallbackWireRequest`/`RegisterCallbackWireResponse` and
   `CONTRACT_RPC_URL`'s expected protocol (this stub assumes a simple JSON
   HTTP RPC; the real contract may expect a Soroban XDR-encoded invocation
   instead, which would change this module significantly — a raw HTTP+JSON
   client is not sufficient for a real Soroban contract call).
4. Sign requests with the actual `relay_signer` key material and signing
   scheme the contract expects — `RELAY_SIGNER_SECRET` is currently sent as a
   bearer token, which is almost certainly not the real auth mechanism for a
   Soroban invocation.
5. Add integration tests against a local Soroban testnet/sandbox once the
   shape is confirmed.