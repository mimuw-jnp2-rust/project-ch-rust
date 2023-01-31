mod lib;
mod p2p;

use crate::p2p::AppBehaviour;
use libp2p::{
    core::upgrade,
    futures::StreamExt,
    mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{Swarm, SwarmBuilder},
    tcp::TokioTcpConfig,
    Transport,
};
use log::{error, info};
use std::time::Duration;
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    select, spawn,
    sync::mpsc,
    time::sleep,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    info!("Peer Id: {}", p2p::PEER_ID.clone());

    let (response_sender, mut response_receiver) = mpsc::unbounded_channel();
    let (init_sender, mut init_receiver) = mpsc::unbounded_channel();

    let auth_keys = Keypair::<X25519Spec>::new()
        .into_authentic(&p2p::KEYS)
        .expect("Can create auth keys.");

    let transport = TokioTcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    let behaviour =
        AppBehaviour::new(lib::App::default(), response_sender, init_sender.clone()).await;

    let mut swarm = SwarmBuilder::new(transport, behaviour, *p2p::PEER_ID)
        .executor(Box::new(|fut| {
            spawn(fut);
        }))
        .build();

    Swarm::listen_on(
        &mut swarm,
        "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("Can get a local socket."),
    )
    .expect("Swarm can be started.");

    spawn(async move {
        sleep(Duration::from_secs(1)).await;
        info!("Sending init event.");
        init_sender.send(true).expect("Can send init event.");
    });

    let mut stdin = BufReader::new(stdin()).lines();

    loop {
        let event = {
            select! {
                line = stdin.next_line() => Some(p2p::EventType::Input(line.expect("Can get line.").expect("Can read line from stdin."))),
                response = response_receiver.recv() => {
                    Some(p2p::EventType::LocalChainResponse(response.expect("Response exists.")))
                },
                _init = init_receiver.recv() => {
                    Some(p2p::EventType::Init)
                }
                event = swarm.select_next_some() => {
                    info!("Unhandled Swarm Event: {:?}", event);
                    None
                },
            }
        };

        if let Some(event) = event {
            match event {
                p2p::EventType::Init => {
                    let peers = p2p::get_list_peers(&swarm);
                    swarm.behaviour_mut().app.genesis();

                    info!("Connected nodes: {}", peers.len());
                    if !peers.is_empty() {
                        let req = p2p::LocalChainRequest {
                            from_peer_id: peers
                                .iter()
                                .last()
                                .expect("At least one peer.")
                                .to_string(),
                        };

                        let json = serde_json::to_string(&req).expect("Can jsonify request.");
                        swarm
                            .behaviour_mut()
                            .floodsub
                            .publish(p2p::CHAIN_TOPIC.clone(), json.as_bytes());
                    }
                }
                p2p::EventType::LocalChainResponse(res) => {
                    let json = serde_json::to_string(&res).expect("Can jsonify response.");
                    swarm
                        .behaviour_mut()
                        .floodsub
                        .publish(p2p::CHAIN_TOPIC.clone(), json.as_bytes());
                }
                p2p::EventType::Input(line) => match line.as_str() {
                    "ls p" => p2p::handle_print_peers(&swarm),
                    cmd if cmd.starts_with("ls accounts") => p2p::handle_print_accounts(&swarm),
                    cmd if cmd.starts_with("ls account") => p2p::handle_print_account(
                        cmd.strip_prefix("ls account").expect("Can strip"),
                        &swarm,
                    ),
                    cmd if cmd.starts_with("ls b") => p2p::handle_print_chain(&swarm),
                    cmd if cmd.starts_with("create account") => {
                        p2p::handle_create_account(&mut swarm)
                    }
                    cmd if cmd.starts_with("transfer ") => p2p::handle_transfer(
                        cmd.strip_prefix("transfer ").expect("Can strip"),
                        &mut swarm,
                    ),
                    _ => error!("Unknown command"),
                },
            }
        }
    }
}
