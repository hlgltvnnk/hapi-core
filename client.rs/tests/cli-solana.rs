use serde_json::json;
use std::{thread::sleep, time::Duration};

mod assert;
mod cmd_utils;
mod solana;

use solana::{fixtures::*, setup::Setup};

#[tokio::test(flavor = "multi_thread")]
async fn solana_cli_works() {
    let t = Setup::new();
}
