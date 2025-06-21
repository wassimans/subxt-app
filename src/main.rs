use subxt::{OnlineClient, PolkadotConfig, dynamic::Value, tx::dynamic as dynamic_call};
use subxt_signer::sr25519::dev;

#[tokio::main]
pub async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // (the port 42069 is specified in the asset-hub-zombienet.toml)
    let api = OnlineClient::<PolkadotConfig>::from_url("ws://127.0.0.1:59650")
        .await
        .map_err(|e| anyhow::anyhow!("RPC error: {e}"))?;
    println!("✅ Connected to dynamic client");

    println!("Available pallets:");
    for p in api.metadata().pallets() {
        println!("  • {}", p.name());
    }

    let alice_pair_signer = dev::alice();
    let alice = Value::from_bytes(alice_pair_signer.public_key());

    const COLLECTION_ID: u128 = 12;
    const NFT_ID: u128 = 234;

    // create a collection with id `12`
    let collection_creation_tx = dynamic_call(
        "Uniques",
        "create",
        vec![Value::u128(COLLECTION_ID), alice.clone()],
    );

    let _collection_creation_events = api
        .tx()
        .sign_and_submit_then_watch_default(&collection_creation_tx, &alice_pair_signer)
        .await
        .map(|e| {
            println!("Collection creation submitted, waiting for transaction to be finalized...");
            e
        })?
        .wait_for_finalized_success()
        .await?;
    println!("🌱 Collection {COLLECTION_ID} created");

    // create an nft in that collection with id `234`
    let nft_creation_tx = dynamic_call(
        "Uniques",
        "mint",
        vec![
            Value::u128(COLLECTION_ID),
            Value::u128(NFT_ID),
            alice.clone(),
        ],
    );
    let _nft_creation_events = api
        .tx()
        .sign_and_submit_then_watch_default(&nft_creation_tx, &alice_pair_signer)
        .await
        .map(|e| {
            println!("NFT creation submitted, waiting for transaction to be finalized...");
            e
        })?
        .wait_for_finalized_success()
        .await?;
    println!("NFT created.");

    // check in storage, that alice is the official owner of the NFT:
    let nft_owner_storage_query = subxt::storage::dynamic(
        "Uniques",
        "Asset",
        vec![Value::u128(COLLECTION_ID), Value::u128(NFT_ID)],
    );
    if let Some(asset_info) = api
        .storage()
        .at_latest()
        .await?
        .fetch(&nft_owner_storage_query)
        .await?
    {
        let asset_value = asset_info.to_value();
        println!("🏷️  Asset info: {asset_value:?}");
    } else {
        println!("❌ Asset not found");
    }

    Ok(())
}
