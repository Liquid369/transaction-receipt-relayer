use std::path::PathBuf;

use ethers::types::Address;
use eyre::Result;
use helios::{client::ClientBuilder, config::networks::Network, prelude::*};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Config {
    consensus_rpc: String,
    untrusted_rpc: String,
    smart_contract_address: String,
    block_number: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let config = envy::from_env::<Config>()?;

    let mut client: Client<FileDB> = ClientBuilder::new()
        .network(Network::MAINNET)
        .consensus_rpc(&config.consensus_rpc)
        .execution_rpc(&config.untrusted_rpc)
        .load_external_fallback()
        .data_dir(PathBuf::from("/tmp/helios"))
        .build()?;

    log::info!(
        "Built client on network \"{}\" with external checkpoint fallbacks",
        Network::MAINNET
    );

    client.start().await?;
    log::info!("client started");

    let filter = ethers::types::Filter::new()
        .select(
            config
                .block_number
                .map(Into::into)
                .unwrap_or(ethers::core::types::BlockNumber::Latest)..,
        )
        .address(config.smart_contract_address.parse::<Address>().unwrap())
        .event("Transfer(address,address,uint256)");

    loop {
        let logs = client.get_logs(&filter).await?;
        log::info!("logs: {:#?}", logs);
    }

    Ok(())
}
