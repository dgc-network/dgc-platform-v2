// Copyright (c) The dgc.network
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::pin::Pin;

use futures::prelude::*;
use sawtooth_sdk::messages::batch::BatchList;
use sawtooth_sdk::messages::client_batch_submit::ClientBatchStatus;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::rest_api::error::RestApiResponseError;

pub const DEFAULT_TIME_OUT: u32 = 300; // Max timeout 300 seconds == 5 minutes

pub trait BatchSubmitter: Send + 'static {
    fn submit_batches(
        &self,
        submit_batches: SubmitBatches,
    ) -> Pin<Box<dyn Future<Output = Result<BatchStatusLink, RestApiResponseError>> + Send>>;

    fn batch_status(
        &self,
        batch_statuses: BatchStatuses,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<BatchStatus>, RestApiResponseError>> + Send>>;

    fn clone_box(&self) -> Box<dyn BatchSubmitter>;
}

impl Clone for Box<dyn BatchSubmitter> {
    fn clone(&self) -> Box<dyn BatchSubmitter> {
        self.clone_box()
    }
}

pub struct SubmitBatches {
    pub batch_list: BatchList,
    pub response_url: Url,
    pub service_id: Option<String>,
}

pub struct BatchStatuses {
    pub batch_ids: Vec<String>,
    pub wait: Option<u32>,
    pub service_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchStatus {
    pub id: String,
    pub invalid_transactions: Vec<HashMap<String, String>>,
    pub status: String,
}

impl BatchStatus {
    pub fn from_proto(proto: &ClientBatchStatus) -> BatchStatus {
        BatchStatus {
            id: proto.get_batch_id().to_string(),
            invalid_transactions: proto
                .get_invalid_transactions()
                .iter()
                .map(|txn| {
                    let mut invalid_transaction_info = HashMap::new();
                    invalid_transaction_info
                        .insert("id".to_string(), txn.get_transaction_id().to_string());
                    invalid_transaction_info
                        .insert("message".to_string(), txn.get_message().to_string());
                    invalid_transaction_info.insert(
                        "extended_data".to_string(),
                        base64::encode(txn.get_extended_data()),
                    );
                    invalid_transaction_info
                })
                .collect(),
            status: format!("{:?}", proto.get_status()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchStatusResponse {
    pub data: Vec<BatchStatus>,
    pub link: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchStatusLink {
    pub link: String,
}
