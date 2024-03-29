use chrono::prelude::*;
use log::{error, info, warn};
use rand::rngs::ThreadRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::vec;

const DIFFICULTY_PREFIX: &str = "00";
const GENESIS_ADDRESS: u64 = 0;
const GENESIS_PUB_KEY: u64 = 1234;
const GENESIS_ACCOUNT: Account = Account {
    address: GENESIS_ADDRESS,
    balance: u64::MAX,
    pub_key: GENESIS_PUB_KEY,
};

const INIT_BALANCE: u64 = 0;

pub type Address = u64;
pub type PrivateKey = u64;
pub type PublicKey = u64;
pub type Signature = u64;

#[derive(Default)]
pub struct Node {
    pub blocks: Vec<Block>,
    pub accounts: HashMap<Address, Account>,
    pub pub_keys: HashMap<Address, PublicKey>,
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub address: Address,
    pub balance: u64,
    pub pub_key: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub id: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: i64,
    pub data: Data,
    pub nonce: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Data {
    Account(Account),
    Transfer(Address, Address, u64, Signature),
}

impl Node {
    pub fn new() -> Self {
        Self {
            blocks: vec![],
            accounts: HashMap::new(),
            pub_keys: HashMap::new(),
        }
    }

    pub fn genesis(&mut self) {
        let genesis_block = Block {
            id: 0,
            previous_hash: String::from("genesis"),
            timestamp: 1665411300,
            data: Data::Account(GENESIS_ACCOUNT.clone()),
            nonce: 420,
            hash: "aeebad4a796fcc2e15dc4c6061b45ed9b373f26adfc798ca7d2d8cc58182718e".to_string(),
        };
        self.pub_keys.insert(GENESIS_ADDRESS, GENESIS_PUB_KEY);
        self.accounts.insert(GENESIS_ADDRESS, GENESIS_ACCOUNT);
        self.blocks.push(genesis_block);
    }

    pub fn add_account(&mut self) -> Account {
        let mut rng = rand::thread_rng();
        let mut account = Account::new(&mut rng);

        loop {
            if !self.accounts.contains_key(&account.address) {
                self.accounts.insert(account.address, account.clone());
                self.pub_keys.insert(account.address, account.pub_key);
                break;
            }

            account = Account::new(&mut rng);
        }

        account
    }

    pub fn try_add_block(&mut self, block: Block) -> bool {
        let latest_block = self.get_last_block();

        if Self::is_block_valid(&block, latest_block) {
            match &block.data {
                Data::Account(account) => {
                    self.accounts.insert(account.address, account.clone());
                    self.pub_keys.insert(account.address, account.pub_key);
                }
                Data::Transfer(..) => {
                    if !self.try_add_transfer(&block.data) {
                        return false;
                    }
                }
            }
            self.blocks.push(block);
            true
        } else {
            error!("Could not add block - invalid.");
            false
        }
    }

    pub fn try_add_transfer(&mut self, transfer: &Data) -> bool {
        if let Data::Transfer(sender, receiver, amount, signature) = transfer {
            if let Some(pub_key) = self.pub_keys.get(sender) {
                if !self.verify_signature(signature, pub_key) {
                    error!("Transfer: signature verification failed");
                    return false;
                }
            } else {
                error!("Transfer: invalid sender address!");
                return false;
            }

            let amount = *amount;
            return if let (Some(acc1), Some(acc2)) =
                (self.accounts.get(sender), self.accounts.get(receiver))
            {
                let balance1 = acc1.balance;
                let balance2 = acc2.balance;
                let pub_key1 = acc1.pub_key;
                let pub_key2 = acc2.pub_key;

                if balance1 < amount {
                    error!("Transfer from: insufficient balance!");
                    return false;
                }
                self.accounts.insert(
                    *sender,
                    Account {
                        address: *sender,
                        balance: balance1 - amount,
                        pub_key: pub_key1,
                    },
                );
                self.accounts.insert(
                    *receiver,
                    Account {
                        address: *receiver,
                        balance: balance2.saturating_add(amount),
                        pub_key: pub_key2,
                    },
                );

                true
            } else {
                error!("Transfer: invalid receiver address!");
                false
            };
        }

        error!("Wrong transfer params!");
        false
    }

    pub fn choose_chain(&mut self, local: Vec<Block>, remote: Vec<Block>) -> Vec<Block> {
        let is_local_valid = self.is_chain_valid(&local);
        let is_remote_valid = self.is_chain_valid(&remote);

        if is_local_valid && is_remote_valid {
            if local.len() >= remote.len() {
                local
            } else {
                remote
            }
        } else if is_local_valid {
            local
        } else if is_remote_valid {
            remote
        } else {
            panic!("Local and remote chains both are invalid!");
        }
    }

    pub fn get_last_block(&self) -> &Block {
        self.blocks.last().expect("There is at least one block")
    }

    fn verify_signature(&self, signature: &Signature, pub_key: &PublicKey) -> bool {
        signature == pub_key
    }

    fn is_chain_valid(&self, chain: &[Block]) -> bool {
        for i in 0..chain.len() {
            if i == 0 {
                continue;
            }
            let first = chain.get(i - 1).expect("First block has to exist.");
            let second = chain.get(i).expect("Second block has to exist.");
            if !Self::is_block_valid(second, first) {
                return false;
            }
        }
        true
    }

    fn is_block_valid(block: &Block, previous_block: &Block) -> bool {
        if block.previous_hash != previous_block.hash {
            warn!("Block with id: {} has wrong previous hash", block.id);
            return false;
        } else if !hash_to_binary_representation(
            &hex::decode(&block.hash).expect("Should decode from hex."),
        )
            .starts_with(DIFFICULTY_PREFIX)
        {
            warn!("Block with id: {} has invalid difficulty.", block.id);
            return false;
        } else if block.id != previous_block.id + 1 {
            warn!(
                "Block with id: {} is not the next block after the latest: {}",
                block.id, previous_block.id
            );
            return false;
        } else if hex::encode(calculate_hash(
            block.id,
            block.timestamp,
            &block.previous_hash,
            &block.data,
            block.nonce,
        )) != block.hash
        {
            warn!("Block with id: {} has invalid hash", block.id);
            return false;
        }
        true
    }
}

impl Block {
    pub fn new(id: u64, previous_hash: String, data: Data) -> Self {
        let now = Utc::now();
        let (nonce, hash) = Block::mine_block(id, now.timestamp(), &previous_hash, &data);
        Self {
            id,
            hash,
            previous_hash,
            timestamp: now.timestamp(),
            data,
            nonce,
        }
    }

    fn mine_block(id: u64, timestamp: i64, previous_hash: &str, data: &Data) -> (u64, String) {
        info!("Mining block ...");
        let mut nonce = 0;

        loop {
            if nonce % 100000 == 0 {
                info!("Nonce: {}", nonce);
            }

            let hash = calculate_hash(id, timestamp, previous_hash, data, nonce);
            let binary_hash = hash_to_binary_representation(&hash);
            if binary_hash.starts_with(DIFFICULTY_PREFIX) {
                info!(
                    "Mined! Nonce: {}, hash: {}, binary_hash: {}",
                    nonce,
                    hex::encode(&hash),
                    binary_hash
                );
                return (nonce, hex::encode(hash));
            }
            nonce += 1;
        }
    }
}

impl Account {
    pub fn new(rng: &mut ThreadRng) -> Self {
        let private_key = rng.gen::<PrivateKey>();
        info!("Private key: {}", private_key);

        Self {
            address: rng.gen::<Address>(),
            balance: INIT_BALANCE,
            pub_key: rng.gen::<PublicKey>(),
        }
    }
}

fn hash_to_binary_representation(hash: &[u8]) -> String {
    let mut rep: String = String::new();
    for c in hash {
        rep.push_str(&format!("{:b}", c));
    }
    rep
}

fn calculate_hash(
    id: u64,
    timestamp: i64,
    previous_hash: &str,
    data: &Data,
    nonce: u64,
) -> Vec<u8> {
    let object = json!({
        "id": id,
        "previous_hash": previous_hash,
        "data": data,
        "timestamp": timestamp,
        "nonce": nonce
    });

    let mut hasher = Sha256::new();
    hasher.update(object.to_string().as_bytes());
    hasher.finalize().as_slice().to_owned()
}

#[cfg(test)]
mod node_tests {
    use super::*;
    use log::Level;

    fn get_genesis_block() -> Block {
        Block {
            id: 0,
            previous_hash: String::from("genesis"),
            timestamp: 1665411300,
            data: Data::Account(GENESIS_ACCOUNT.clone()),
            nonce: 420,
            hash: "aeebad4a796fcc2e15dc4c6061b45ed9b373f26adfc798ca7d2d8cc58182718e".to_string(),
        }
    }

    fn get_first_block() -> Block {
        Block {
            id: 1,
            previous_hash: "aeebad4a796fcc2e15dc4c6061b45ed9b373f26adfc798ca7d2d8cc58182718e"
                .to_string(),
            timestamp: 1665411301,
            data: Data::Account(Account {
                address: 1,
                balance: INIT_BALANCE,
                pub_key: 1111,
            }),
            nonce: 38656,
            hash: "00003a55bc3e237053bcc5444b589a093c596a4d8d0b2ec6b3a2177f4bdeb42f".to_string(),
        }
    }

    #[test]
    fn creates_genesis_block() {
        let mut node = Node::new();
        let genesis_block = get_genesis_block();

        node.genesis();

        assert_eq!(node.blocks.len(), 1);
        assert_eq!(node.blocks.first().unwrap(), &genesis_block);
    }

    #[ignore]
    #[test]
    fn validates_first_block() {
        let mut node = Node::new();
        let first_block = get_first_block();

        node.genesis();
        node.try_add_block(first_block.clone());

        assert_eq!(node.blocks.len(), 2);
        assert_eq!(node.blocks.get(1).unwrap(), &first_block);
    }

    #[test]
    fn does_not_validate_with_wrong_previous_hash() {
        let mut node = Node::new();
        let mut first_block = get_first_block();
        first_block.previous_hash.replace_range(0..1, "f");

        testing_logger::setup();

        node.genesis();
        node.try_add_block(first_block);

        assert_eq!(node.blocks.len(), 1);
        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 2);
            assert_eq!(
                captured_logs[0].body,
                "Block with id: 1 has wrong previous hash"
            );
            assert_eq!(captured_logs[0].level, Level::Warn);
            assert_eq!(captured_logs[1].body, "Could not add block - invalid.");
            assert_eq!(captured_logs[1].level, Level::Error);
        })
    }

    #[test]
    fn does_not_validate_with_wrong_difficulty() {
        let mut node = Node::new();
        let mut first_block = get_first_block();
        first_block.hash.replace_range(0..2, "0f");

        testing_logger::setup();

        node.genesis();
        node.try_add_block(first_block);

        assert_eq!(node.blocks.len(), 1);
        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 2);
            assert_eq!(
                captured_logs[0].body,
                "Block with id: 1 has invalid difficulty."
            );
            assert_eq!(captured_logs[0].level, Level::Warn);
            assert_eq!(captured_logs[1].body, "Could not add block - invalid.");
            assert_eq!(captured_logs[1].level, Level::Error);
        })
    }

    #[test]
    fn does_not_validate_with_wrong_id() {
        let mut node = Node::new();
        let mut first_block = get_first_block();
        first_block.id = 2;

        testing_logger::setup();

        node.genesis();
        node.try_add_block(first_block);

        assert_eq!(node.blocks.len(), 1);
        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 2);
            assert_eq!(
                captured_logs[0].body,
                "Block with id: 2 is not the next block after the latest: 0"
            );
            assert_eq!(captured_logs[0].level, Level::Warn);
            assert_eq!(captured_logs[1].body, "Could not add block - invalid.");
            assert_eq!(captured_logs[1].level, Level::Error);
        })
    }

    #[test]
    fn does_not_validate_with_wrong_hash() {
        let mut node = Node::new();
        let mut first_block = get_first_block();
        first_block.data = Data::Account(Account {
            address: 1,
            balance: 0,
            pub_key: 2222,
        });
        testing_logger::setup();

        node.genesis();
        node.try_add_block(first_block);

        assert_eq!(node.blocks.len(), 1);
        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 2);
            assert_eq!(captured_logs[0].body, "Block with id: 1 has invalid hash");
            assert_eq!(captured_logs[0].level, Level::Warn);
            assert_eq!(captured_logs[1].body, "Could not add block - invalid.");
            assert_eq!(captured_logs[1].level, Level::Error);
        })
    }

    #[ignore]
    #[test]
    fn validates_chain() {
        let node = Node::new();
        let is_valid =
            node.is_chain_valid((vec![get_genesis_block(), get_first_block()]).as_slice());

        assert!(is_valid);
    }

    #[test]
    fn does_not_validate_chain() {
        let node = Node::new();
        let is_valid = node.is_chain_valid(
            (vec![get_genesis_block(), get_genesis_block(), get_first_block()]).as_slice(),
        );

        assert!(!is_valid);
    }
}
