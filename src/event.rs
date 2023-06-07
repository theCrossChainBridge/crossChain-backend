use ethers::{prelude::EthEvent, types::{U256, Address}};

#[derive(Clone, Debug, EthEvent)]
pub struct Stake {
    #[ethevent(indexed)]
    pub account: Address,
    pub token_addr: Address,
    pub amount: U256,
}