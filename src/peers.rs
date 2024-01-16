/// Get a list of peers of a mastodon instance
pub async fn get_peers(mastodon_url: &str) -> Result<Vec<String>, reqwest::Error> {
    let response = reqwest::get(format!("{}/api/v1/instance/peers", mastodon_url)).await?;
    response.json().await
}
