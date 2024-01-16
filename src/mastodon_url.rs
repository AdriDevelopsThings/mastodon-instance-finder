use std::time::Duration;

use regex::Regex;
use reqwest::redirect::Policy;

/// Get the url to the mastodon instance by it's domain
/// It tries to check if there is a redirect on the webfinger endpoint
pub async fn get_mastodon_url(domain: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(5))
        .redirect(Policy::none())
        .build()
        .unwrap()
        .get(format!("https://{domain}/.well-known/webfinger"))
        .send()
        .await?;
    if let Some(location) = response.headers().get("Location") {
        if let Some(captures) = Regex::new(r"https://(.+)/.well-known/webfinger")
            .unwrap()
            .captures(location.to_str().unwrap())
        {
            let capture = captures.get(1).unwrap().as_str();
            return Ok(format!("https://{capture}"));
        }
    }
    Ok(format!("https://{domain}"))
}
