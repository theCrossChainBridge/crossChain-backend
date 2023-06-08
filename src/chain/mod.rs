use dotenv::dotenv;
use ethers::prelude::*;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::{
    env,
    fs::OpenOptions,
    io::{BufReader, Read},
    sync::Arc,
};
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

mod abi;
use abi::Bridge;
mod event;
use event::Stake;

#[derive(Deserialize)]
pub struct Conf {
    pub contract_address: Net,
}

#[derive(Deserialize)]
pub struct Net {
    pub sepolia: String,
    pub mumbai: String,
}

pub async fn run(ws: &mut WebSocketStream<TcpStream>, msg: String) -> eyre::Result<()> {
    // Read the configuration file and deserialize it into a Conf struct
    let mut buf = BufReader::new(OpenOptions::new().read(true).open("config.toml")?);
    let mut conf = String::new();
    buf.read_to_string(&mut conf).unwrap();

    let config: Conf = toml::from_str(&conf).unwrap();

    // Connect to the Ethereum client and get the contract address from the configuration
    let eth_client = get_eth_client().await;
    let eth_address = config.contract_address.sepolia.parse::<Address>().unwrap();

    // Create a contract instance from the address and client with relevant ABI
    let eth_contract = Bridge::new(eth_address, Arc::new(eth_client));

    // Subscribe to the Stake event from the Ethereum contract
    let events = eth_contract.event::<Stake>();
    let mut event_stream = events.subscribe_with_meta().await?;

    // Connect to the Matic client and get the contract address from the configuration
    let matic_provider = get_matic_provider().await;
    let matic_address: H160 = config.contract_address.mumbai.parse::<Address>().unwrap();

    // Create a client with the wallet as the signer middleware
    let wallet = get_wallet().await;
    let client = SignerMiddleware::new(matic_provider, wallet);

    // Create a contract instance from the address and client with relevant ABI
    let matic_contract = Bridge::new(matic_address, Arc::new(client));

    // Listen to the event stream and mint tokens on Matic network
    while let Some(Ok((event, _meta))) = event_stream.next().await {
        println!("Stake Event: {event:?}");

        let account: Address = event.account;
        let token_addr: Address = event.token_addr;
        let amount: U256 = event.amount;
        let given_account = msg.parse::<Address>().unwrap();

        if account == given_account {
            let tx = matic_contract
                .mint(account, token_addr, amount)
                .send()
                .await?
                .log_msg("Pending hash")
                .await?;
            let matic_hash = tx.unwrap().transaction_hash;
            ws.send(Message::Text(format!("{:x}", matic_hash)))
                .await
                .unwrap();
        }
    }

    Ok(())
}

async fn get_eth_client() -> Provider<Ws> {
    dotenv().ok();
    let sepolia: String = env::var("SEPOLIA_RPC_URL").expect("SEPOLIA_RPC_URL must be set");
    let rpc_url: &str = sepolia.as_str();

    Provider::<Ws>::connect(rpc_url).await.unwrap()
}

async fn get_matic_provider() -> Provider<Http> {
    dotenv().ok();
    let mumbai: String = env::var("MUMBAI_RPC_URL").expect("MUMBAI_RPC_URL must be set");
    let rpc_url: &str = mumbai.as_str();

    Provider::<Http>::try_from(rpc_url).unwrap()
}

async fn get_wallet() -> LocalWallet {
    let private_key: String = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");
    private_key
        .as_str()
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(Chain::PolygonMumbai)
}
