use mastodon_instance_finder::start_fetching;

#[tokio::main]
async fn main() {
    start_fetching().await;
}
