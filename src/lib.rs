use log::{error, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

const DIFFICULTY_PREFIX: &str = "00";

pub struct App {
    pub blocks: Vec<Block>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub id: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: i64,
    pub data: String,
    pub nonce: u64,
}

fn hash_to_string_representation(hash: &[u8]) -> String {
    let mut rep: String = String::default();
    for c in hash {
        rep.push_str(&format!("{:b}", c));
    }
    rep
}

fn calculate_hash(id: u64, timestamp: i64, previous_hash: &str, data: &str, nonce: u64) -> Vec<u8> {
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

impl App {
    pub fn default() -> Self {
        Self { blocks: vec![] }
    }

    pub fn genesis(&mut self) {
        let genesis_block = Block {
            id: 0,
            previous_hash: String::from("genesis"),
            timestamp: 1665411300,
            data: String::from("genesis"),
            nonce: 420,
            hash: "aeebad4a796fcc2e15dc4c6061b45ed9b373f26adfc798ca7d2d8cc58182718e".to_string(),
        };
        self.blocks.push(genesis_block);
    }

    pub fn try_add_block(&mut self, block: Block) {
        let latest_block = self
            .blocks
            .last()
            .expect("There should be at least one block.");
        if self.is_block_valid(&block, latest_block) {
            self.blocks.push(block);
        } else {
            error!("Could not add block - invalid.");
        }
    }

    fn is_block_valid(&self, block: &Block, previous_block: &Block) -> bool {
        if block.previous_hash != previous_block.hash {
            warn!("Block with id: {} has wrong previous hash", block.id);
            return false;
        } else if !hash_to_string_representation(
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

#[cfg(test)]
mod app_tests {
    use super::*;
    use log::Level;

    fn get_genesis_block() -> Block {
        Block {
            id: 0,
            previous_hash: String::from("genesis"),
            timestamp: 1665411300,
            data: String::from("genesis"),
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
            data: String::from("first_block"),
            nonce: 78321,
            hash: "0000590a7f2735c5ebf696401385dc3f76e33cd4dc3bd7ceeff7be992ada1c98".to_string(),
        }
    }

    #[test]
    fn creates_genesis_block() {
        let mut app = App::default();
        let genesis_block = get_genesis_block();

        app.genesis();

        assert_eq!(app.blocks.len(), 1);
        assert_eq!(app.blocks.first().unwrap(), &genesis_block);
    }

    #[test]
    fn validates_first_block() {
        let mut app = App::default();
        let first_block = get_first_block();

        app.genesis();
        app.try_add_block(first_block.clone());

        assert_eq!(app.blocks.len(), 2);
        assert_eq!(app.blocks.get(1).unwrap(), &first_block);
    }

    #[test]
    fn does_not_validate_with_wrong_previous_hash() {
        let mut app = App::default();
        let mut first_block = get_first_block();
        first_block.previous_hash.replace_range(0..1, "f");

        testing_logger::setup();

        app.genesis();
        app.try_add_block(first_block);

        assert_eq!(app.blocks.len(), 1);
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
        let mut app = App::default();
        let mut first_block = get_first_block();
        first_block.hash.replace_range(0..2, "0f");

        testing_logger::setup();

        app.genesis();
        app.try_add_block(first_block);

        assert_eq!(app.blocks.len(), 1);
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
        let mut app = App::default();
        let mut first_block = get_first_block();
        first_block.id = 2;

        testing_logger::setup();

        app.genesis();
        app.try_add_block(first_block);

        assert_eq!(app.blocks.len(), 1);
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
        let mut app = App::default();
        let mut first_block = get_first_block();
        first_block.data = "ala ma kota".to_string();
        testing_logger::setup();

        app.genesis();
        app.try_add_block(first_block);

        assert_eq!(app.blocks.len(), 1);
        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 2);
            assert_eq!(captured_logs[0].body, "Block with id: 1 has invalid hash");
            assert_eq!(captured_logs[0].level, Level::Warn);
            assert_eq!(captured_logs[1].body, "Could not add block - invalid.");
            assert_eq!(captured_logs[1].level, Level::Error);
        })
    }
}
