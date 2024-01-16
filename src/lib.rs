use std::{
    env,
    path::{Path, PathBuf},
    process::exit,
    sync::Arc,
};

use ::nodeinfo::NodeInfoOwned;
use mastodon_url::get_mastodon_url;
use nodeinfo::nodeinfo;
use peers::get_peers;
use serde::Serialize;
use tokio::{
    fs::{create_dir, File},
    io::AsyncWriteExt,
    sync::{mpsc, Mutex, Semaphore},
};

mod mastodon_url;
mod nodeinfo;
mod peers;

const INVALID_DOMAINS: &[&str] = &["activitypub-troll.cf"];

/// return if the expression returns an error, otherwise return the value
macro_rules! return_fail {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(_e) => {
                return;
            }
        }
    };
}

fn is_valid_domain(domain: &str) -> bool {
    for invalid_domain in INVALID_DOMAINS.iter() {
        if domain.ends_with(invalid_domain) {
            return false;
        }
    }
    true
}

fn get_target_dir() -> PathBuf {
    Path::new(&env::var("TARGET_DIR").unwrap_or_else(|_| "output".to_string())).to_path_buf()
}

fn get_domain_path(domain: &str) -> PathBuf {
    get_target_dir().join(format!("{domain}.json"))
}

/// Save an instance to the target directory
async fn save_instance(instance: Instance) {
    let mut file = File::create(get_domain_path(&instance.domain))
        .await
        .unwrap();
    file.write_all(serde_json::to_string(&instance).unwrap().as_bytes())
        .await
        .unwrap();
}

#[derive(Serialize)]
struct Instance {
    domain: String,
    mastodon_url: String,
    nodeinfo: NodeInfoOwned,
}

pub async fn start_fetching() {
    // All new domains will put into the channel
    let (tx, mut rx) = mpsc::channel::<String>(100);
    // A list of already fetched domains
    let already_fetched: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    // Count of instances
    let instances: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    // Count of users
    let users: Arc<Mutex<i64>> = Arc::new(Mutex::new(0));
    // Semaphore to block too much requests at the same time
    let semaphore = Arc::new(Semaphore::new(30));

    let target_dir = get_target_dir();
    // create the target directory if it doesn't exist
    if !target_dir.exists() {
        create_dir(target_dir).await.unwrap();
    }

    // start with one domain
    tx.send("chaos.social".to_string()).await.unwrap();

    while let Some(domain) = rx.recv().await {
        // clone arc's and sender
        let instances = instances.clone();
        let users = users.clone();
        let already_fetched = already_fetched.clone();
        let tx = tx.clone();

        // acquire permit
        let _permit = semaphore.clone().acquire_owned().await.unwrap();
        tokio::spawn(async move {
            // get the real mastodon url of the domain
            let url = return_fail!(get_mastodon_url(&domain).await);
            let info = return_fail!(nodeinfo(&url).await);
            if info.software.name != "mastodon" {
                return;
            }
            if let Some(info_users) = info.usage.users.total {
                let mut users_lock = users.lock().await;
                *users_lock += info_users
            }
            (*instances.lock().await) += 1;
            println!(
                "Found {domain: <26} Users: {} Instances: {} Queue: {} Already fetched: {}",
                users.lock().await,
                instances.lock().await,
                tx.max_capacity() - tx.capacity(),
                already_fetched.lock().await.len()
            );

            // save instance to target directory
            let cloned_url = url.clone();
            tokio::spawn(async move {
                save_instance(Instance {
                    domain,
                    mastodon_url: cloned_url,
                    nodeinfo: info,
                })
                .await;
            });

            // find new peers of this domain
            let tx = tx.clone();
            let already_fetched = already_fetched.clone();
            tokio::spawn(async move {
                let peers = return_fail!(get_peers(&url).await);
                for peer in peers {
                    let mut l = already_fetched.lock().await;
                    if !l.contains(&peer) && is_valid_domain(&peer) {
                        l.push(peer.clone());
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            tx.send(peer.clone()).await.unwrap();
                        });
                    }
                }
                // if the queue is empty the finding is finished
                if tx.capacity() == tx.max_capacity() {
                    exit(0);
                }
            });
            drop(_permit);
        });
    }
}
