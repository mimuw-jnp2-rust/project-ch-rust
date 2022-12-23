use serde::{Serialize, Deserialize};

pub struct App {
    pub blocks: Vec<Block>
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Block {
    pub id: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: i64,
    pub data: String,
    pub nonce: u64,
}

impl App {
    pub fn new() -> Self {
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
}

#[cfg(test)]
mod app_tests {
    use super::*;

    #[test]
    fn creates_genesis_block() {
        let mut app: App = App::new();
        app.genesis();

        let expected_genesis = Block {
            id: 0,
            previous_hash: String::from("genesis"),
            timestamp: 1665411300,
            data: String::from("genesis"),
            nonce: 420,
            hash: "aeebad4a796fcc2e15dc4c6061b45ed9b373f26adfc798ca7d2d8cc58182718e".to_string(),
        };

        assert_eq!(app.blocks.len(), 1);
        assert_eq!(app.blocks.first().unwrap(), &expected_genesis);
    }
}
