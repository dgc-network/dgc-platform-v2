-- Copyright 2019 Cargill Incorporated
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.
-- -----------------------------------------------------------------------------

ALTER TABLE agent ADD COLUMN IF NOT EXISTS service_id TEXT;

ALTER TABLE associated_agent ADD COLUMN IF NOT EXISTS service_id TEXT;

ALTER TABLE grid_property_definition ADD COLUMN IF NOT EXISTS service_id TEXT;

ALTER TABLE grid_schema ADD COLUMN IF NOT EXISTS service_id TEXT;

ALTER TABLE organization ADD COLUMN IF NOT EXISTS service_id TEXT;

ALTER TABLE product ADD COLUMN IF NOT EXISTS service_id TEXT;

ALTER TABLE product_property_value ADD COLUMN IF NOT EXISTS service_id TEXT;

ALTER TABLE property ADD COLUMN IF NOT EXISTS service_id TEXT;

ALTER TABLE proposal ADD COLUMN IF NOT EXISTS service_id TEXT;

ALTER TABLE record ADD COLUMN IF NOT EXISTS service_id TEXT;

ALTER TABLE reported_value ADD COLUMN IF NOT EXISTS service_id TEXT;

ALTER TABLE reporter ADD COLUMN IF NOT EXISTS service_id TEXT;

DROP VIEW reported_value_reporter_to_agent_metadata;
DROP VIEW reporter_to_agent_metadata;

CREATE VIEW reporter_to_agent_metadata
AS
  SELECT id,
         property_name,
         record_id,
         public_key,
         authorized,
         reporter_index,
         metadata,
         reporter_end_block_num,
         service_id
  FROM   (SELECT Row_number()
                   OVER (
                     partition BY id
                     ORDER BY agent_end_block_num) AS RowNum,
                 *
          FROM   (SELECT reporter.id,
                         reporter.property_name,
                         reporter.record_id,
                         reporter.reporter_index,
                         reporter.authorized,
                         reporter.public_key,
                         reporter.end_block_num AS "reporter_end_block_num",
                         agent.end_block_num    AS "agent_end_block_num",
                         agent.metadata,
                         agent.service_id
                  FROM   reporter
                         LEFT JOIN agent
                                ON reporter.public_key = agent.public_key
                                   AND reporter.end_block_num <=
                                       agent.end_block_num) AS
                 join_tables) X
  WHERE  rownum = 1;

CREATE VIEW reported_value_reporter_to_agent_metadata
AS
  SELECT id,
         property_name,
         record_id,
         reporter_index,
         timestamp,
         data_type,
         bytes_value,
         boolean_value,
         number_value,
         string_value,
         enum_value,
         struct_values,
         lat_long_value,
         public_key,
         authorized,
         metadata,
         reported_value_end_block_num,
         reporter_end_block_num,
         service_id
  FROM   (SELECT Row_number()
                   OVER (
                     partition BY id
                     ORDER BY reporter_end_block_num) AS RowNum,
                 *
          FROM   (SELECT reported_value.id,
                         reported_value.property_name,
                         reported_value.record_id,
                         reported_value.reporter_index,
                         reported_value.timestamp,
                         reported_value.data_type,
                         reported_value.bytes_value,
                         reported_value.boolean_value,
                         reported_value.number_value,
                         reported_value.string_value,
                         reported_value.enum_value,
                         reported_value.struct_values,
                         reported_value.lat_long_value,
                         reported_value.end_block_num AS
                         "reported_value_end_block_num",
                         reporter_to_agent_metadata.reporter_end_block_num,
                         reporter_to_agent_metadata.public_key,
                         reporter_to_agent_metadata.authorized,
                         reporter_to_agent_metadata.metadata,
                         reported_value.service_id
                  FROM   reported_value
                         LEFT JOIN reporter_to_agent_metadata
                                ON reported_value.record_id =
                                   reporter_to_agent_metadata.record_id
                                   AND reported_value.property_name =
                                       reporter_to_agent_metadata.property_name
                                   AND reported_value.reporter_index =
                                       reporter_to_agent_metadata.reporter_index
                                   AND reported_value.end_block_num <=
                                       reporter_to_agent_metadata.reporter_end_block_num) AS
                    join_tables) X
    WHERE  rownum = 1;
