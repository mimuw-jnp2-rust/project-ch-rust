# Blockchain Rust Project (ch-rust)

![](https://c.pxhere.com/photos/bb/21/chain_rust_iron_metal_macro_rusty-1087626.jpg!d)

---

## Authors
- Antoni Koszowski (@akoszowski on GitHub)

## Description
ch-rust is going to be an application based on simple blockchain implementation 
enabling users to connect to the given node operator in p2p network and live preview
currently processed transactions.

Idea is based on the [article](https://blog.logrocket.com/how-to-build-a-blockchain-in-rust/).

## Features
- Simple blockchain implementation
- Node operators logs preview via console
- UI enabling to connect to the given node operator
- UI live preview of the currently processed transactions

## Extra features
- Externally Owned Accounts + API for their creation
- On-chain transactions
- Reasonable consensus algorithm
- UI - creating EOA, generating private key
- UI - sending transactions to friends

## Plan
1. In the first part I am going to implement the backend logic: simple blockchain implementation, node operators logs preview via console.

2. In the second part I am going to add UI enabling to connect to the given node operator and live preview of transactions.

__Extra__: If there would be enough time I plan to realize features described in _Extra features_ section.

## Libraries
- [Tokio](https://tokio.rs/)
- [Libp2p](https://crates.io/crates/libp2p)
- optionally [Yew](https://yew.rs/)
