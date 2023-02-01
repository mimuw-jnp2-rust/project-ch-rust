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

## Project summary
#### Added features
- [x] Blockchain and nodes implementation
- [x] Nodes console log preview and commands
- [x] Possibility to generate externally owned accounts
- [x] Private, public key generation, signature authorization (concept)
- [x] On-chain transaction
- [x] Client available, but not integrated with the `Blockchain` part
- [x] Project splitted into workspaces
#### Missing features (possible future enhancements)
- [ ] UI - connecting to given node
- [ ] UI - live preview of transactions
- [ ] UI - interface to send transactions to other addresses

#### Few remarks here:
 - having started project with bit outdated dependencies versions, 
moving forward it was hard to bump the up without major code refactoring, what I wanted to avoid
 - only later did I realize that `Yew` and `Tokio` is not enough for successful e2e integration,
for it to happen I should have added `Warp` (or sth similar) as well, however it was not compatible with outdated`libp2p`;
here lessons learned: managing dependencies may be troublesome + it is better to be up to date with the dependencies :)
 -  implementing handlers in `Warp` I would be able to create endpoints leveraging existing backend logic then used by 
appropriate client's requests
 - `Swarm` spawns p2p nodes with dynamic port binding, thus it would be hard connecting to a specific one, 
not really having a good idea here how to effectively resolve this problem
 - Even though, there is still plenty of improvements or new features that could be introduced,
I am pretty satisfied with the final results - I see how much I ve learned this semester, here in project getting acquainted with
`libp2p`, `tokio`, `yew` and researching `warp`. Still though, there is a lot of things to learn and advance on.
Hopefully my Rustacean journey just begins :)

### Running ch-rust:
```bash
cargo build
```
Open few terminals (e.g. 3), in each of them run:
```bash
RUST_LOG=info cargo run
```
Feel free to experiment with commands:
- `ls b` - list mined blocks
- `ls p` - list peers in network
- `ls accounts` - list information about all accounts
- `ls account <address>` - list information about account with given address
- `create account` - create new account, get the  __<address, balance, pub_key>__; see __<private_key>__ printed to the console
- `transfer {"Transfer":[1,2,3,4]}` - transfers _<from, to, amount, signature>_

#### Running dummy UI client
```bash
trunk serve
```

### First part notes
As already mentioned first part is based, in a high degree, on the [article](https://blog.logrocket.com/how-to-build-a-blockchain-in-rust/).

`Block` and `App` logic was slightly improved and now is covered with unit tests -> see Github CI. Also goal of getting acquainted 
with `libp2p` has been realized.

## Libraries
- [Tokio](https://tokio.rs/)
- [Libp2p](https://crates.io/crates/libp2p)
- optionally [Yew](https://yew.rs/)
