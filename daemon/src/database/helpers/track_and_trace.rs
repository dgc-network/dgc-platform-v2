/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use super::models::{
    AssociatedAgent, NewAssociatedAgent, NewProperty, NewProposal, NewRecord, NewReportedValue,
    NewReporter, Property, Proposal, Record, ReportedValueReporterToAgentMetadata, Reporter,
};
use super::schema::{
    associated_agent, grid_property_definition, property, proposal, record, reported_value,
    reported_value_reporter_to_agent_metadata, reporter,
};
use super::MAX_COMMIT_NUM;

use diesel::{
    dsl::{insert_into, update},
    pg::PgConnection,
    prelude::*,
    result::Error::NotFound,
    QueryResult,
};

pub fn insert_associated_agents(
    conn: &PgConnection,
    agents: &[NewAssociatedAgent],
) -> QueryResult<()> {
    for agent in agents {
        update_associated_agent_end_commit_num(
            conn,
            &agent.record_id,
            agent.service_id.as_deref(),
            &agent.role,
            &agent.agent_id,
            agent.start_commit_num,
        )?;
    }

    insert_into(associated_agent::table)
        .values(agents)
        .execute(conn)
        .map(|_| ())
}

pub fn update_associated_agent_end_commit_num(
    conn: &PgConnection,
    record_id: &str,
    service_id: Option<&str>,
    role: &str,
    agent_id: &str,
    current_commit_num: i64,
) -> QueryResult<()> {
    let update = update(associated_agent::table);

    if let Some(service_id) = service_id {
        update
            .filter(
                associated_agent::record_id
                    .eq(record_id)
                    .and(associated_agent::role.eq(role))
                    .and(associated_agent::agent_id.eq(agent_id))
                    .and(associated_agent::service_id.eq(service_id))
                    .and(associated_agent::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(associated_agent::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    } else {
        update
            .filter(
                associated_agent::record_id
                    .eq(record_id)
                    .and(associated_agent::role.eq(role))
                    .and(associated_agent::agent_id.eq(agent_id))
                    .and(associated_agent::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(associated_agent::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }
}

pub fn list_associated_agents(
    conn: &PgConnection,
    record_ids: &[String],
    service_id: Option<&str>,
) -> QueryResult<Vec<AssociatedAgent>> {
    let mut query = associated_agent::table
        .into_boxed()
        .select(associated_agent::all_columns)
        .filter(
            associated_agent::end_commit_num
                .eq(MAX_COMMIT_NUM)
                .and(associated_agent::record_id.eq_any(record_ids)),
        );

    if let Some(service_id) = service_id {
        query = query.filter(associated_agent::service_id.eq(service_id));
    } else {
        query = query.filter(associated_agent::service_id.is_null());
    }
    query.load::<AssociatedAgent>(conn)
}

pub fn insert_properties(conn: &PgConnection, properties: &[NewProperty]) -> QueryResult<()> {
    for property in properties {
        update_property_end_commit_num(
            conn,
            &property.name,
            property.service_id.as_deref(),
            &property.record_id,
            property.start_commit_num,
        )?;
    }

    insert_into(property::table)
        .values(properties)
        .execute(conn)
        .map(|_| ())
}

pub fn update_property_end_commit_num(
    conn: &PgConnection,
    name: &str,
    service_id: Option<&str>,
    record_id: &str,
    current_commit_num: i64,
) -> QueryResult<()> {
    let update = update(property::table);

    if let Some(service_id) = service_id {
        update
            .filter(
                property::name
                    .eq(name)
                    .and(property::record_id.eq(record_id))
                    .and(property::service_id.eq(service_id))
                    .and(property::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(property::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    } else {
        update
            .filter(
                property::name
                    .eq(name)
                    .and(property::record_id.eq(record_id))
                    .and(property::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(property::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }
}

pub fn insert_proposals(conn: &PgConnection, proposals: &[NewProposal]) -> QueryResult<()> {
    for proposal in proposals {
        update_proposal_end_commit_num(
            conn,
            &proposal.record_id,
            proposal.service_id.as_deref(),
            &proposal.receiving_agent,
            &proposal.role,
            proposal.start_commit_num,
        )?;
    }

    insert_into(proposal::table)
        .values(proposals)
        .execute(conn)
        .map(|_| ())
}

pub fn update_proposal_end_commit_num(
    conn: &PgConnection,
    record_id: &str,
    service_id: Option<&str>,
    receiving_agent: &str,
    role: &str,
    current_commit_num: i64,
) -> QueryResult<()> {
    let update = update(proposal::table);

    if let Some(service_id) = service_id {
        update
            .filter(
                proposal::record_id
                    .eq(record_id)
                    .and(proposal::receiving_agent.eq(receiving_agent))
                    .and(proposal::service_id.eq(service_id))
                    .and(proposal::role.eq(role))
                    .and(proposal::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(proposal::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    } else {
        update
            .filter(
                proposal::record_id
                    .eq(record_id)
                    .and(proposal::receiving_agent.eq(receiving_agent))
                    .and(proposal::role.eq(role))
                    .and(proposal::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(proposal::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }
}

pub fn list_proposals(
    conn: &PgConnection,
    record_ids: &[String],
    service_id: Option<&str>,
) -> QueryResult<Vec<Proposal>> {
    let mut query = proposal::table
        .into_boxed()
        .select(proposal::all_columns)
        .filter(
            proposal::end_commit_num
                .eq(MAX_COMMIT_NUM)
                .and(proposal::record_id.eq_any(record_ids)),
        );

    if let Some(service_id) = service_id {
        query = query.filter(proposal::service_id.eq(service_id));
    } else {
        query = query.filter(proposal::service_id.is_null());
    }

    query.load::<Proposal>(conn)
}

pub fn insert_records(conn: &PgConnection, records: &[NewRecord]) -> QueryResult<()> {
    for record in records {
        update_record_end_commit_num(
            conn,
            &record.record_id,
            record.service_id.as_deref(),
            record.start_commit_num,
        )?;
    }

    insert_into(record::table)
        .values(records)
        .execute(conn)
        .map(|_| ())
}

pub fn update_record_end_commit_num(
    conn: &PgConnection,
    record_id: &str,
    service_id: Option<&str>,
    current_commit_num: i64,
) -> QueryResult<()> {
    let update = update(record::table);

    if let Some(service_id) = service_id {
        update
            .filter(
                record::record_id
                    .eq(record_id)
                    .and(record::service_id.eq(service_id))
                    .and(record::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(record::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    } else {
        update
            .filter(
                record::record_id
                    .eq(record_id)
                    .and(record::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(record::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }
}

pub fn fetch_record(conn: &PgConnection, record_id: &str) -> QueryResult<Option<Record>> {
    record::table
        .select(record::all_columns)
        .filter(
            record::record_id
                .eq(record_id)
                .and(record::end_commit_num.eq(MAX_COMMIT_NUM)),
        )
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn list_records(conn: &PgConnection, service_id: Option<&str>) -> QueryResult<Vec<Record>> {
    let mut query = record::table
        .into_boxed()
        .select(record::all_columns)
        .filter(record::end_commit_num.eq(MAX_COMMIT_NUM));

    if let Some(service_id) = service_id {
        query = query.filter(record::service_id.eq(service_id));
    } else {
        query = query.filter(record::service_id.is_null());
    }
    query.load::<Record>(conn)
}

pub fn insert_reported_values(conn: &PgConnection, values: &[NewReportedValue]) -> QueryResult<()> {
    for value in values {
        update_reported_value_end_commit_num(
            conn,
            &value.property_name,
            value.service_id.as_deref(),
            &value.record_id,
            value.start_commit_num,
        )?;
    }

    insert_into(reported_value::table)
        .values(values)
        .execute(conn)
        .map(|_| ())
}

pub fn update_reported_value_end_commit_num(
    conn: &PgConnection,
    property_name: &str,
    service_id: Option<&str>,
    record_id: &str,
    current_commit_num: i64,
) -> QueryResult<()> {
    let update = update(reported_value::table);

    if let Some(service_id) = service_id {
        update
            .filter(
                reported_value::record_id
                    .eq(record_id)
                    .and(reported_value::service_id.eq(service_id))
                    .and(reported_value::property_name.eq(property_name))
                    .and(reported_value::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(reported_value::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    } else {
        update
            .filter(
                reported_value::record_id
                    .eq(record_id)
                    .and(reported_value::property_name.eq(property_name))
                    .and(reported_value::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(reported_value::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }
}

pub fn insert_reporters(conn: &PgConnection, reporters: &[NewReporter]) -> QueryResult<()> {
    for reporter in reporters {
        update_reporter_end_commit_num(
            conn,
            &reporter.property_name,
            reporter.service_id.as_deref(),
            &reporter.record_id,
            &reporter.public_key,
            reporter.start_commit_num,
        )?;
    }

    insert_into(reporter::table)
        .values(reporters)
        .execute(conn)
        .map(|_| ())
}

pub fn update_reporter_end_commit_num(
    conn: &PgConnection,
    property_name: &str,
    service_id: Option<&str>,
    record_id: &str,
    public_key: &str,
    current_commit_num: i64,
) -> QueryResult<()> {
    let update = update(reporter::table);

    if let Some(service_id) = service_id {
        update
            .filter(
                reporter::record_id
                    .eq(record_id)
                    .and(reporter::property_name.eq(property_name))
                    .and(reporter::service_id.eq(service_id))
                    .and(reporter::public_key.eq(public_key))
                    .and(reporter::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(reporter::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    } else {
        update
            .filter(
                reporter::record_id
                    .eq(record_id)
                    .and(reporter::property_name.eq(property_name))
                    .and(reporter::public_key.eq(public_key))
                    .and(reporter::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .set(reporter::end_commit_num.eq(current_commit_num))
            .execute(conn)
            .map(|_| ())
    }
}

pub fn fetch_property_with_data_type(
    conn: &PgConnection,
    record_id: &str,
    property_name: &str,
    service_id: Option<&str>,
) -> QueryResult<Option<(Property, Option<String>)>> {
    let mut query = property::table
        .into_boxed()
        .left_join(
            record::table.on(property::record_id
                .eq(record::record_id)
                .and(property::end_commit_num.eq(record::end_commit_num))),
        )
        .left_join(
            grid_property_definition::table.on(record::schema
                .eq(grid_property_definition::schema_name)
                .and(property::name.eq(grid_property_definition::name))
                .and(property::end_commit_num.eq(record::end_commit_num))),
        )
        .filter(
            property::name
                .eq(property_name)
                .and(property::record_id.eq(record_id))
                .and(property::end_commit_num.eq(MAX_COMMIT_NUM)),
        );

    if let Some(service_id) = service_id {
        query = query.filter(property::service_id.eq(service_id));
    } else {
        query = query.filter(property::service_id.is_null());
    }
    query
        .select((
            property::all_columns,
            grid_property_definition::data_type.nullable(),
        ))
        .first::<(Property, Option<String>)>(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn fetch_reported_value_reporter_to_agent_metadata(
    conn: &PgConnection,
    record_id: &str,
    property_name: &str,
    commit_height: Option<i64>,
) -> QueryResult<Option<ReportedValueReporterToAgentMetadata>> {
    let commit_height = commit_height.unwrap_or(MAX_COMMIT_NUM);
    reported_value_reporter_to_agent_metadata::table
        .filter(
            reported_value_reporter_to_agent_metadata::property_name
                .eq(property_name)
                .and(reported_value_reporter_to_agent_metadata::record_id.eq(record_id))
                .and(
                    reported_value_reporter_to_agent_metadata::reported_value_end_commit_num
                        .eq(commit_height),
                ),
        )
        .first(conn)
        .map(Some)
        .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
}

pub fn list_properties_with_data_type(
    conn: &PgConnection,
    record_ids: &[String],
    service_id: Option<&str>,
) -> QueryResult<Vec<(Property, Option<String>)>> {
    let mut query = property::table
        .into_boxed()
        .left_join(
            record::table.on(property::record_id
                .eq(record::record_id)
                .and(property::end_commit_num.eq(record::end_commit_num))),
        )
        .left_join(
            grid_property_definition::table.on(record::schema
                .eq(grid_property_definition::schema_name)
                .and(property::name.eq(grid_property_definition::name))
                .and(property::end_commit_num.eq(record::end_commit_num))),
        )
        .filter(
            property::record_id
                .eq_any(record_ids)
                .and(property::end_commit_num.eq(MAX_COMMIT_NUM)),
        );

    if let Some(service_id) = service_id {
        query = query.filter(property::service_id.eq(service_id));
    } else {
        query = query.filter(property::service_id.is_null());
    }
    query
        .select((
            property::all_columns,
            grid_property_definition::data_type.nullable(),
        ))
        .load::<(Property, Option<String>)>(conn)
}

pub fn list_reporters(
    conn: &PgConnection,
    record_id: &str,
    property_name: &str,
) -> QueryResult<Vec<Reporter>> {
    reporter::table
        .filter(
            reporter::property_name
                .eq(property_name)
                .and(reporter::record_id.eq(record_id))
                .and(reporter::end_commit_num.eq(MAX_COMMIT_NUM)),
        )
        .load::<Reporter>(conn)
}

pub fn list_reported_value_reporter_to_agent_metadata(
    conn: &PgConnection,
    record_id: &str,
    property_name: &str,
) -> QueryResult<Vec<ReportedValueReporterToAgentMetadata>> {
    reported_value_reporter_to_agent_metadata::table
        .filter(
            reported_value_reporter_to_agent_metadata::property_name
                .eq(property_name)
                .and(reported_value_reporter_to_agent_metadata::record_id.eq(record_id))
                .and(
                    reported_value_reporter_to_agent_metadata::reported_value_end_commit_num
                        .le(MAX_COMMIT_NUM),
                ),
        )
        .load::<ReportedValueReporterToAgentMetadata>(conn)
}
