use serde_json::json;
use std::{thread::sleep, time::Duration};

mod assert;
mod evm;
mod solana;

use solana::{fixtures::*, setup::Setup};

#[tokio::test(flavor = "multi_thread")]
async fn solana_cli_works() {
    let t = Setup::new();
}
