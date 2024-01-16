# mastodon-instance-finder
Find all mastodon instance and save their nodeinfo json to a directory.

## Build
Build it with `cargo`:
```cargo build --release```

## Run
Run: ```target/release/mastodon-instance-finder```
You can set the environment variable `TARGET_DIR`, otherwise the json files will be saved to `target`.