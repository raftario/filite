use anyhow::Result;
use argon2::Config;
use rand::Rng;
use tokio::task;

// TODO: Allow custom configuration
async fn hash(password: Vec<u8>) -> Result<String> {
    let config = Config::default();
    Ok(task::spawn_blocking(move || {
        let salt: [u8; 16] = rand::thread_rng().gen();
        argon2::hash_encoded(&password, &salt[..], &config)
    })
    .await??)
}

async fn verify(encoded: String, password: Vec<u8>) -> Result<bool> {
    Ok(task::spawn_blocking(move || argon2::verify_encoded(&encoded, &password)).await??)
}
