use {
    anchor_lang::AccountSerialize,
    hapi_core::{
        client::solana::{test_helpers::create_test_tx, InstructionData},
        HapiCoreNetwork,
    },
    hapi_indexer::{IndexingCursor, PushData, SOLANA_BATCH_SIZE},
    mockito::{Matcher, Server, ServerGuard},
    serde_json::{json, Value},
    solana_account_decoder::{UiAccount, UiAccountEncoding},
    solana_sdk::{account::Account, pubkey::Pubkey},
    solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta,
    std::str::FromStr,
};

use super::{RpcMock, TestBatch, TestData};

pub const PROGRAM_ID: &str = "39WzZqJgkK2QuQxV9jeguKRgHE65Q3HywqPwBzdrKn2B";
pub const REPORTER: &str = "C7DNJUKfDVpL9ZZqLnVTG1adj4Yu46JgDB6hiTdMEktX";
pub const CASE: &str = "DTDk9GEQoVibTuHmTfDUwHehkH4WYd5fpawPfayGRVdi";
pub const ADDRESS: &str = "WN4cDdcxEEzCVyaFEuG4zzJB6QNqrahtfYpSeeecrmC";
pub const ASSET: &str = "5f2iaDyv4yzTudiNc1XR2EkEW5NtVbfZpqmjZ3fhFtaX";

pub struct SolanaMock {
    server: ServerGuard,
}

impl RpcMock for SolanaMock {
    fn get_contract_address() -> String {
        PROGRAM_ID.to_string()
    }

    fn get_network() -> HapiCoreNetwork {
        HapiCoreNetwork::Solana
    }

    fn get_hashes() -> [String; 6] {
        ["3rsZaASe9nEWoSudhqSXTCYSdWzE4ynNdmdwa4DWmpttkjRbsrPy292YUN1gm7LSm9zKh9X6oCUJoML7uuJEWZM5".to_string(), 
        "KZsY26mofenqWeTRK2KpboNNWBsxZvauSmCbnehFHK5ALzksZEi6Jci91KNENec8NXv4p9Ksq68FmKAJDBTCKiT".to_string(),
        "4dxiswkbCh4bE1wDevwN1jUa1cLozDyb78WPdFoqr7UJn3RuC5PHpCbJ7zQAXQTxcvXqrWYVtYGnP3Q8kHgg8FM".to_string(),
        "5PwVsyu5jxHKJUp2qps8ZwARyh3jm6edCw62m8NAZVDWMngntMEpgCojyVDATn1qK3VaLZJ3LZzQw94mBcgaYoLz".to_string(),
        "BMt6636zZ7QbR9sjmwMcTFqEZaKNWmrMhbxMxTdZqyni2sNv3xQoqZ6Z2h3qaGQhW5aZutPNkdLJ94jt2gMGHJs".to_string(),
        "4RcKC84uwEVPx9qw1toe4W1JrTeyG76pWg6GN5x3FFHrCt9wb6QSDhtSCaZ4kpZXkyz4QcYubYGfecGtWsXtTNmM".to_string()]
    }

    fn initialize() -> Self {
        let mut server = Server::new();

        let response = json!({
           "jsonrpc": "2.0",
           "result": { "feature-set": 289113172, "solana-core": "1.16.7" },
           "id": 1
        });

        server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&response.to_string())
            .match_body(Matcher::PartialJson(json!({
                "method":"getVersion",
            })))
            .create();

        Self { server }
    }

    fn get_mock_url(&self) -> String {
        self.server.url()
    }

    fn fetching_jobs_mock(&mut self, batches: &[TestBatch], cursor: &IndexingCursor) {
        let mut before = None;
        let until = match cursor {
            IndexingCursor::None => None,
            IndexingCursor::Transaction(tx) => Some(tx.to_string()),
            IndexingCursor::Block(_) => panic!("Invalid cursor"),
        };

        for batch in batches {
            let signatures: Vec<Value> = batch
                .iter()
                .map(|data| {
                    json!({
                        "signature": data.hash,
                        "slot": 100,
                    })
                })
                .collect();

            self.mock_batches(signatures, &before, &until);

            before = batch.last().map(|data| data.hash.clone());
        }

        self.mock_batches(vec![], &before, &until);
    }

    fn processing_jobs_mock(&mut self, batch: &TestBatch) {
        for event in batch {
            self.mock_transaction(event.name.to_string(), &event.hash);
            self.mock_accounts(event);
        }
    }
}

impl SolanaMock {
    fn get_transaction(name: String, hash: &str) -> EncodedConfirmedTransactionWithStatusMeta {
        // TODO: what about asset?
        let account_keys = vec![
            String::from(PROGRAM_ID),
            String::default(),
            String::default(),
            String::from(REPORTER),
            String::from(CASE),
            String::from(ADDRESS),
        ];

        create_test_tx(
            &vec![(
                name.as_str(),
                InstructionData::Raw(String::from("Some data")),
            )],
            hash.to_string(),
            account_keys,
        )
    }

    fn mock_batches(
        &mut self,
        signatures: Vec<Value>,
        before: &Option<String>,
        until: &Option<String>,
    ) {
        let response = json!({
            "jsonrpc": "2.0",
            "result": signatures,
            "id": 1
        });

        self.server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&response.to_string())
            .match_body(Matcher::PartialJson(json!({
                "method": "getSignaturesForAddress",
                "params": [ PROGRAM_ID,
                {
                  "limit": SOLANA_BATCH_SIZE,
                  "until" : until,
                  "before" : before,
                  "commitment" : "confirmed"
                }],
            })))
            .create();
    }

    fn mock_transaction(&mut self, name: String, hash: &str) {
        let response = json!({
           "jsonrpc": "2.0",
           "result": json!(SolanaMock::get_transaction(name, hash)),
           "id": 1
        });

        self.server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&response.to_string())
            .match_body(Matcher::PartialJson(json!({
                "method": "getTransaction",
                "params": [
                    hash,
                    "json"
                  ]
            })))
            .create();
    }

    fn get_account_data(payload_data: PushData) -> (Pubkey, Vec<u8>) {
        let mut data = Vec::new();

        let address = match payload_data {
            PushData::Address(address) => {
                hapi_core_solana::Address {
                    version: 1,
                    bump: 255,
                    network: Pubkey::default(),
                    address: encode_address(&address.address),
                    category: address.category.into(),
                    risk_score: address.risk,
                    case_id: address.case_id.as_u128(),
                    reporter_id: address.reporter_id.as_u128(),
                    confirmations: 0,
                }
                .try_serialize(&mut data)
                .expect("Failed to serialize address");

                ADDRESS
            }
            PushData::Asset(asset) => {
                hapi_core_solana::Asset {
                    version: 1,
                    bump: 255,
                    network: Pubkey::default(),
                    // TODO: encode asset id
                    address: encode_address(&asset.address),
                    id: [0u8; 64],
                    category: asset.category.into(),
                    risk_score: asset.risk,
                    case_id: asset.case_id.as_u128(),
                    reporter_id: asset.reporter_id.as_u128(),
                    confirmations: 0,
                }
                .try_serialize(&mut data)
                .expect("Failed to serialize asset");

                ASSET
            }
            PushData::Case(case) => {
                hapi_core_solana::Case {
                    version: 1,
                    bump: 255,
                    network: Pubkey::default(),
                    id: case.id.as_u128(),
                    name: case.name,
                    reporter_id: case.reporter_id.as_u128(),
                    status: case.status.into(),
                    url: case.url,
                }
                .try_serialize(&mut data)
                .expect("Failed to serialize case");

                CASE
            }
            PushData::Reporter(reporter) => {
                hapi_core_solana::Reporter {
                    version: 1,
                    bump: 255,
                    network: Pubkey::default(),
                    id: reporter.id.as_u128(),
                    name: reporter.name,
                    account: Pubkey::from_str(reporter.account.as_str())
                        .expect("Invalid reporter address"),
                    role: reporter.role.into(),
                    status: reporter.status.into(),
                    unlock_timestamp: reporter.unlock_timestamp,
                    url: reporter.url,
                    stake: reporter.stake.into(),
                }
                .try_serialize(&mut data)
                .expect("Failed to serialize reporter");

                REPORTER
            }
        };

        (Pubkey::from_str(address).expect("Invalid address"), data)
    }

    fn mock_accounts(&mut self, test_data: &TestData) {
        if let Some(payload_data) = &test_data.data {
            let (address, data) = SolanaMock::get_account_data(payload_data.clone());

            let account = Account {
                lamports: 100,
                data,
                owner: Pubkey::from_str(PROGRAM_ID).expect("Invalid program id"),
                executable: false,
                rent_epoch: 123,
            };

            let encoded_account = UiAccount::encode(
                &address,
                &account,
                UiAccountEncoding::Base64Zstd,
                None,
                None,
            );

            let response = json!({
               "jsonrpc": "2.0",
               "result": {
                "context": { "apiVersion": "1.16.17", "slot": 252201350 },
                "value": json!(encoded_account),
               },
               "id": 1
            });

            println!("RESPONCE : {}", response.to_string());

            self.server
                .mock("POST", "/")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(&response.to_string())
                .match_body(Matcher::PartialJson(json!({
                    "method": "getAccountInfo",
                    "params": [
                        address.to_string(),
                    ]
                })))
                .create();
        }
    }
}

pub fn encode_address(address: &str) -> [u8; 64] {
    let mut res = [0u8; 64];
    let bytes = address.as_bytes();
    res[..bytes.len()].copy_from_slice(bytes);

    res
}
