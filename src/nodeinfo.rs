use nodeinfo::NodeInfoOwned;
use reqwest::StatusCode;

#[derive(Debug)]
pub enum NodeInfoError {
    InvalidStatus(StatusCode),
    Request(reqwest::Error),
    NodeInfo(nodeinfo::NodeInfoError),
    WrongContentType,
}

pub async fn nodeinfo(mastodon_url: &str) -> Result<NodeInfoOwned, NodeInfoError> {
    let response = reqwest::get(format!("{mastodon_url}/nodeinfo/2.0"))
        .await
        .map_err(NodeInfoError::Request)?;
    if response.status() != StatusCode::OK {
        return Err(NodeInfoError::InvalidStatus(response.status()));
    }
    let content_type = response
        .headers()
        .get("Content-Type")
        .ok_or(NodeInfoError::WrongContentType)?
        .to_str()
        .unwrap();
    if !content_type.starts_with("application/json") {
        return Err(NodeInfoError::WrongContentType);
    }
    Ok(nodeinfo::deserialize(&response.text().await.unwrap())
        .map_err(NodeInfoError::NodeInfo)?
        .to_owned())
}
