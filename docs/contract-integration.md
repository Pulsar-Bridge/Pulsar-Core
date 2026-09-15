# Contract integration (`register_callback`)

`src/contracts/mod.rs` defines a `ContractClient` trait and a
`SorobanContractClient` HTTP implementation that calls
`pulsar-core-contracts`' `register_callback()` as the trusted `relay_signer`.