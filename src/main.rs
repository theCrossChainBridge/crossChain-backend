use dotenv::dotenv;
use ethers::prelude::*;
use serde::Deserialize;
use std::{
    env,
    fs::OpenOptions,
    io::{BufReader, Read},
    sync::Arc,
};

mod abi;
use abi::Bridge;

mod event;
use event::Stake;

// Defined in 'config.toml'
#[derive(Deserialize)]
pub struct Conf {
    sepolia: String,
    mumbai: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Read the configuration file and deserialize it into a Conf struct
    let mut buf = BufReader::new(OpenOptions::new().read(true).open("config.toml")?);
    let mut conf = String::new();
    buf.read_to_string(&mut conf).unwrap();

    let config: Conf = toml::from_str(&conf).unwrap();

    // Connect to the Ethereum client and get the contract address from the configuration
    let eth_client = get_eth_client().await;
    let eth_address = config.sepolia.parse::<Address>().unwrap();

    // Create a contract instance from the address and client with relevant ABI
    let eth_contract = Bridge::new(eth_address, Arc::new(eth_client));

    // Subscribe to the Stake event from the Ethereum contract
    let events = eth_contract.event::<Stake>();
    let mut event_stream = events.subscribe().await.unwrap();

    // Connect to the Matic client and get the contract address from the configuration
    let matic_provider = get_matic_provider().await;
    let matic_address: H160 = config.mumbai.parse::<Address>().unwrap();

    // Create a client with the wallet as the signer middleware
    let wallet = get_wallet().await;
    let client = SignerMiddleware::new(matic_provider, wallet);

    // Create a contract instance from the address and client with relevant ABI
    let matic_contract = Bridge::new(matic_address, Arc::new(client));

    // Listen to the event stream and mint tokens on Matic network
    while let Some(Ok(stake)) = event_stream.next().await {
        println!("Stake Event: {stake:?}");

        let account: Address = stake.account;
        let token_addr: Address = stake.token_addr;
        let amount: U256 = stake.amount;

        let _tx = matic_contract
            .mint(account, token_addr, amount)
            .send()
            .await?
            .log_msg("Pending hash")
            .await?;
    }

    Ok(())
}

async fn get_eth_client() -> Provider<Ws> {
    dotenv().ok();
    let sepolia: String = env::var("Sepolia_RPC_URL").expect("Sepolia RPC URL must be set");
    let rpc_url: &str = sepolia.as_str();

    Provider::<Ws>::connect(rpc_url).await.unwrap()
}

async fn get_matic_provider() -> Provider<Http> {
    dotenv().ok();
    let mumbai: String = env::var("Mumbai_RPC_URL").expect("Mumbai RPC URL must be set");
    let rpc_url: &str = mumbai.as_str();

    Provider::<Http>::try_from(rpc_url).unwrap()
}

async fn get_wallet() -> LocalWallet {
    let private_key: String = env::var("PrivateKey").expect("Private Key must be set");
    private_key
        .as_str()
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(Chain::PolygonMumbai)
}
