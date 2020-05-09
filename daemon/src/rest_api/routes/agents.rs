// Copyright 2019 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::database::{helpers as db, models::Agent};
use crate::rest_api::{
    error::RestApiResponseError, routes::DbExecutor, AcceptServiceIdParam, AppState, QueryServiceId,
};

use actix::{Handler, Message, SyncContext};
use actix_web::{web, HttpRequest, HttpResponse};
use sawtooth_sdk::messages::batch::BatchList;
use crate::submitter::{BatchStatusResponse, BatchStatuses, SubmitBatches, DEFAULT_TIME_OUT};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentSlice {
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub roles: Vec<String>,
    pub metadata: JsonValue,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl AgentSlice {
    pub fn from_agent(agent: &Agent) -> Self {
        Self {
            public_key: agent.public_key.clone(),
            org_id: agent.org_id.clone(),
            active: agent.active,
            roles: agent.roles.clone(),
            metadata: agent.metadata.clone(),
            service_id: agent.service_id.clone(),
        }
    }
}

struct ListAgents {
    service_id: Option<String>,
}

impl Message for ListAgents {
    type Result = Result<Vec<AgentSlice>, RestApiResponseError>;
}

impl Handler<ListAgents> for DbExecutor {
    type Result = Result<Vec<AgentSlice>, RestApiResponseError>;

    fn handle(&mut self, msg: ListAgents, _: &mut SyncContext<Self>) -> Self::Result {
        let fetched_agents =
            db::get_agents(&*self.connection_pool.get()?, msg.service_id.as_deref())?
                .iter()
                .map(|agent| AgentSlice::from_agent(agent))
                .collect::<Vec<AgentSlice>>();

        Ok(fetched_agents)
    }
}

pub async fn list_agents(
    state: web::Data<AppState>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(ListAgents {
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|agents| HttpResponse::Ok().json(agents))
}

struct FetchAgent {
    public_key: String,
    service_id: Option<String>,
}

impl Message for FetchAgent {
    type Result = Result<AgentSlice, RestApiResponseError>;
}

impl Handler<FetchAgent> for DbExecutor {
    type Result = Result<AgentSlice, RestApiResponseError>;

    fn handle(&mut self, msg: FetchAgent, _: &mut SyncContext<Self>) -> Self::Result {
        let fetched_agent = match db::get_agent(
            &*self.connection_pool.get()?,
            &msg.public_key,
            msg.service_id.as_deref(),
        )? {
            Some(agent) => AgentSlice::from_agent(&agent),
            None => {
                return Err(RestApiResponseError::NotFoundError(format!(
                    "Could not find agent with public key: {}",
                    msg.public_key
                )));
            }
        };

        Ok(fetched_agent)
    }
}

pub async fn fetch_agent(
    state: web::Data<AppState>,
    public_key: web::Path<String>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(FetchAgent {
            public_key: public_key.into_inner(),
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|agent| HttpResponse::Ok().json(agent))
}

//use crate::error::CliError;
//use crate::rest_api::http::submit_batches;
use crate::rest_api::transaction::{pike_batch_builder, PIKE_NAMESPACE};
use grid_sdk::{
    protocol::pike::payload::{Action, CreateAgentAction, PikePayloadBuilder, UpdateAgentAction},
    protos::IntoProto,
};
/*
pub fn create_agent(
    url: &str,
    key: Option<String>,
    wait: u64,
    create_agent: CreateAgentAction,
    service_id: Option<String>,
) -> Result<HttpResponse, RestApiResponseError> {
    let payload = PikePayloadBuilder::new()
        .with_action(Action::CreateAgent)
        .with_create_agent(create_agent)
        .build()
        .map_err(|err| RestApiResponseError::UserError(format!("{}", err)))?;

    let batch_list = pike_batch_builder(key)
        .add_transaction(
            &payload.into_proto()?,
            &[PIKE_NAMESPACE.to_string()],
            &[PIKE_NAMESPACE.to_string()],
        )?
        .create_batch_list();

    submit_batches(url, wait, &batch_list, service_id.as_deref());

    Ok(HttpResponse::Ok().json("status:done"))
}
*/
pub async fn create_agent(
    req: HttpRequest,
    body: web::Bytes,
    state: web::Data<AppState>,
    query_service_id: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    let payload = PikePayloadBuilder::new()
        .with_action(Action::CreateAgent)
        .with_create_agent(create_agent)
        .build()
        .map_err(|err| RestApiResponseError::UserError(format!("{}", err)))?;

    let batch_list: BatchList = match protobuf::parse_from_bytes(&*body) {
        Ok(batch_list) => batch_list,
        Err(err) => {
            return Err(RestApiResponseError::BadRequest(format!(
                "Protobuf message was badly formatted. {}",
                err.to_string()
            )));
        }
    };

    let response_url = req.url_for_static("batch_statuses")?;

    state
        .batch_submitter
        .submit_batches(SubmitBatches {
            batch_list,
            response_url,
            service_id: query_service_id.into_inner().service_id,
        })
        .await
        .map(|link| HttpResponse::Ok().json(link))
}

pub async fn update_agent(
    req: HttpRequest,
    body: web::Bytes,
    state: web::Data<AppState>,
    query_service_id: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    let batch_list: BatchList = match protobuf::parse_from_bytes(&*body) {
        Ok(batch_list) => batch_list,
        Err(err) => {
            return Err(RestApiResponseError::BadRequest(format!(
                "Protobuf message was badly formatted. {}",
                err.to_string()
            )));
        }
    };

    let response_url = req.url_for_static("batch_statuses")?;

    state
        .batch_submitter
        .submit_batches(SubmitBatches {
            batch_list,
            response_url,
            service_id: query_service_id.into_inner().service_id,
        })
        .await
        .map(|link| HttpResponse::Ok().json(link))
}
/*
pub fn update_agent(
    url: &str,
    key: Option<String>,
    wait: u64,
    update_agent: UpdateAgentAction,
    service_id: Option<String>,
) -> Result<HttpResponse, RestApiResponseError> {
    let payload = PikePayloadBuilder::new()
        .with_action(Action::UpdateAgent)
        .with_update_agent(update_agent)
        .build()
        .map_err(|err| RestApiResponseError::UserError(format!("{}", err)))?;

    let batch_list = pike_batch_builder(key)
        .add_transaction(
            &payload.into_proto()?,
            &[PIKE_NAMESPACE.to_string()],
            &[PIKE_NAMESPACE.to_string()],
        )?
        .create_batch_list();

    submit_batches(url, wait, &batch_list, service_id.as_deref());

    Ok(HttpResponse::Ok().json("status:done"))
}
*/