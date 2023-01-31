use libp2p::{
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity,
    mdns::{Mdns, MdnsEvent},
    swarm::{NetworkBehaviourEventProcess, Swarm},
    NetworkBehaviour, PeerId,
};
use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::sync::mpsc;
use crate::lib::{App, Address, Block, Data};

pub static KEYS: Lazy<identity::Keypair> = Lazy::new(identity::Keypair::generate_ed25519);
pub static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
pub static CHAIN_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("chains"));
pub static BLOCK_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("blocks"));

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainResponse {
    pub blocks: Vec<Block>,
    pub receiver: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalChainRequest {
    pub from_peer_id: String,
}

pub enum EventType {
    LocalChainResponse(ChainResponse),
    Input(String),
    Init,
}

#[derive(NetworkBehaviour)]
pub struct AppBehaviour {
    pub floodsub: Floodsub,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub response_sender: mpsc::UnboundedSender<ChainResponse>,
    #[behaviour(ignore)]
    pub init_sender: mpsc::UnboundedSender<bool>,
    #[behaviour(ignore)]
    pub app: App,
}

impl AppBehaviour {
    pub async fn new(
        app: App,
        response_sender: mpsc::UnboundedSender<ChainResponse>,
        init_sender: mpsc::UnboundedSender<bool>,
    ) -> Self {
        let mut behaviour = Self {
            app,
            floodsub: Floodsub::new(*PEER_ID),
            mdns: Mdns::new(Default::default())
                .await
                .expect("Can created mdns."),
            response_sender,
            init_sender,
        };
        behaviour.floodsub.subscribe(CHAIN_TOPIC.clone());
        behaviour.floodsub.subscribe(BLOCK_TOPIC.clone());

        behaviour
    }
}

impl NetworkBehaviourEventProcess<FloodsubEvent> for AppBehaviour {
    fn inject_event(&mut self, event: FloodsubEvent) {
        if let FloodsubEvent::Message(msg) = event {
            if let Ok(res) = serde_json::from_slice::<ChainResponse>(&msg.data) {
                if res.receiver == PEER_ID.to_string() {
                    info!("Response from {}:", msg.source);
                    res.blocks.iter().for_each(|r| info!("{:?}", r));

                    self.app.blocks = self.app.choose_chain(self.app.blocks.clone(), res.blocks);
                }
            } else if let Ok(res) = serde_json::from_slice::<LocalChainRequest>(&msg.data) {
                info!("Sending local chain to {}", msg.source.to_string());
                let peer_id = res.from_peer_id;
                if PEER_ID.to_string() == peer_id {
                    if let Err(e) = self.response_sender.send(ChainResponse {
                        blocks: self.app.blocks.clone(),
                        receiver: msg.source.to_string(),
                    }) {
                        error!("Error sending response via channel, {}", e);
                    }
                }
            } else if let Ok(block) = serde_json::from_slice::<Block>(&msg.data) {
                info!("Received new block from {}", msg.source.to_string());
                self.app.try_add_block(block);
            }
        }
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for AppBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(discovered_list) => {
                for (peer, _addr) in discovered_list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(expired_list) => {
                for (peer, _addr) in expired_list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
            }
        }
    }
}

pub fn get_list_peers(swarm: &Swarm<AppBehaviour>) -> Vec<String> {
    info!("Discovered Peers:");
    let nodes = swarm.behaviour().mdns.discovered_nodes();
    let mut unique_peers = HashSet::new();
    for peer in nodes {
        unique_peers.insert(peer);
    }
    unique_peers.iter().map(|p| p.to_string()).collect()
}

pub fn handle_print_peers(swarm: &Swarm<AppBehaviour>) {
    let peers = get_list_peers(swarm);
    peers.iter().for_each(|p| info!("{}", p));
}

pub fn handle_print_accounts(swarm: &Swarm<AppBehaviour>) {
    info!("Accounts:");
    let pretty_json = serde_json::to_string_pretty(&swarm.behaviour().app.accounts)
        .expect("Can jsonify accounts");
    info!("{}", pretty_json);
}

pub fn handle_print_account(cmd: &str, swarm: &Swarm<AppBehaviour>) {
    if let Ok(address) = serde_json::from_str::<Address>(cmd) {
        if let Some(account) = swarm.behaviour().app.accounts.get(&address) {
            let pretty_json = serde_json::to_string_pretty(account).expect("Can jsonify account.");
            info!("Account:");
            info!("{}", pretty_json);
        } else {
            info!("No account with address: <{:?}>", address);
        }
    } else {
        error!("ls account: error parsing");
    }
}

pub fn handle_print_chain(swarm: &Swarm<AppBehaviour>) {
    info!("Local Blockchain:");
    let pretty_json =
        serde_json::to_string_pretty(&swarm.behaviour().app.blocks).expect("Can jsonify blocks.");
    info!("{}", pretty_json);
}

pub fn handle_create_block(data: Data, swarm: &mut Swarm<AppBehaviour>) {
    let behaviour = swarm.behaviour_mut();
    let latest_block = behaviour
        .app
        .blocks
        .last()
        .expect("There is at least one block");
    let block = Block::new(latest_block.id + 1, latest_block.hash.clone(), data);
    let json = serde_json::to_string(&block).expect("Can jsonify request.");
    behaviour.app.blocks.push(block);
    info!("Broadcasting new block");
    behaviour
        .floodsub
        .publish(BLOCK_TOPIC.clone(), json.as_bytes());
}

pub fn handle_create_account(swarm: &mut Swarm<AppBehaviour>) {
    let behaviour = swarm.behaviour_mut();
    let new_account = behaviour.app.add_account();
    info!("Creating new account with address: {}", new_account.address);

    let data = Data::Account(new_account);
    handle_create_block(data, swarm);
}

pub fn handle_transfer(cmd: &str, swarm: &mut Swarm<AppBehaviour>) {
    let behaviour = swarm.behaviour_mut();
    info!("Sending transfer");

    if let Ok(data) = serde_json::from_str::<Data>(cmd) {
        if behaviour.app.try_add_transfer(&data) {
            handle_create_block(data, swarm);
        }
    } else {
        error!("Transfer: error parsing!");
    }
}
