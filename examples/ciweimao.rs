use anyhow::Result;

use novel_api::{CiweimaoClient, Client};

#[tokio::main]
async fn main() -> Result<()> {
    let client = CiweimaoClient::new().await?;

    client.login("".to_string(), "".to_string()).await?;

    let user_info = client.user_info().await?.unwrap();
    println!("{user_info:#?}");

    Ok(())
}
