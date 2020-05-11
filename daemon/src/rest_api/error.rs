// Copyright 2019 Bitwise IO, Inc.
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

use crate::database::DatabaseError;

use actix::MailboxError;
use actix_web::error::{PayloadError, UrlGenerationError};
use actix_web::{
    error::{Error as ActixError, ResponseError},
    HttpResponse,
};
use futures::future::{Future, TryFutureExt};
use std::error::Error;

use std::fmt;

use grid_sdk::protos;
use sawtooth_sdk::signing;
use std::io;
//use sawtooth_sdk::messaging::stream::SendError;

#[derive(Debug)]
pub enum RestApiServerError {
    StartUpError(String),
    StdError(std::io::Error),
}

impl From<std::io::Error> for RestApiServerError {
    fn from(err: std::io::Error) -> RestApiServerError {
        RestApiServerError::StdError(err)
    }
}

impl Error for RestApiServerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RestApiServerError::StartUpError(_) => None,
            RestApiServerError::StdError(err) => Some(err),
        }
    }
}

impl fmt::Display for RestApiServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RestApiServerError::StartUpError(e) => write!(f, "Start-up Error: {}", e),
            RestApiServerError::StdError(e) => write!(f, "Std Error: {}", e),
        }
    }
}

#[derive(Debug)]
pub enum RestApiResponseError {
    BadRequest(String),
    SawtoothConnectionError(String),
    SawtoothValidatorResponseError(String),
    RequestHandlerError(String),
    DatabaseError(String),
    NotFoundError(String),
    UserError(String),
    //IoError(io::Error),
    //ProtobufError(protobuf::ProtobufError),
    //SigningError(signing::Error),
    //SendError(SendError),
    GridProtoError(protos::ProtoConversionError),
    //SabreProtoError(sabre_sdk::protos::ProtoConversionError),
}

impl Error for RestApiResponseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RestApiResponseError::BadRequest(_) => None,
            RestApiResponseError::SawtoothConnectionError(_) => None,
            RestApiResponseError::SawtoothValidatorResponseError(_) => None,
            RestApiResponseError::RequestHandlerError(_) => None,
            RestApiResponseError::DatabaseError(_) => None,
            RestApiResponseError::NotFoundError(_) => None,
            RestApiResponseError::UserError(_) => None,
            //RestApiResponseError::IoError(err) => Some(err),
            //RestApiResponseError::ProtobufError(err) => Some(err),
            //RestApiResponseError::SigningError(err) => Some(err),
            //RestApiResponseError::SendError(err) => Some(err),
            RestApiResponseError::GridProtoError(err) => Some(err),
            //RestApiResponseError::SabreProtoError(err) => Some(err),
        }
    }
}

impl fmt::Display for RestApiResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RestApiResponseError::BadRequest(ref s) => write!(f, "Bad Request: {}", s),
            RestApiResponseError::SawtoothConnectionError(ref s) => {
                write!(f, "Zmq Connection Error: {}", s)
            }
            RestApiResponseError::SawtoothValidatorResponseError(ref s) => {
                write!(f, "Sawtooth Validator Response Error: {}", s)
            }
            RestApiResponseError::RequestHandlerError(ref s) => {
                write!(f, "Request Handler Error Error: {}", s)
            }
            RestApiResponseError::NotFoundError(ref s) => write!(f, "Not Found Error: {}", s),
            RestApiResponseError::DatabaseError(ref s) => write!(f, "Database Error: {}", s),
            RestApiResponseError::UserError(ref err) => write!(f, "Error: {}", err),
            //RestApiResponseError::IoError(ref err) => write!(f, "IoError: {}", err),
            //RestApiResponseError::ProtobufError(ref err) => write!(f, "ProtobufError: {}", err),
            //RestApiResponseError::SigningError(ref err) => write!(f, "SigningError: {}", err),
            //RestApiResponseError::SendError(ref err) => write!(f, "SendError: {}", err),
            RestApiResponseError::GridProtoError(ref err) => write!(f, "Grid Proto Error: {}", err),
            //RestApiResponseError::SabreProtoError(ref err) => write!(f, "Sabre Proto Error: {}", err),
        }
    }
}

// impl ResponseError trait allows to convert our errors into http responses with appropriate data
impl RestApiResponseError {
    pub fn future_box(self) -> Box<dyn Future<Output = Result<HttpResponse, ActixError>>> {
        match self {
            RestApiResponseError::BadRequest(ref message) => {
                Box::new(HttpResponse::BadRequest().json(message).into_future())
            }
            RestApiResponseError::SawtoothConnectionError(ref message) => Box::new(
                HttpResponse::ServiceUnavailable()
                    .json(message)
                    .into_future(),
            ),
            RestApiResponseError::DatabaseError(ref message) => Box::new(
                HttpResponse::ServiceUnavailable()
                    .json(message)
                    .into_future(),
            ),
            RestApiResponseError::NotFoundError(ref message) => {
                Box::new(HttpResponse::NotFound()
                    .json(message)
                    .into_future())
            }
            _ => Box::new(
                HttpResponse::InternalServerError()
                    .json("Internal Server Error")
                    .into_future(),
            ),
        }
    }
}
// impl ResponseError trait allows to convert our errors into http responses with appropriate data
impl ResponseError for RestApiResponseError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            RestApiResponseError::BadRequest(ref message) => {
                HttpResponse::BadRequest().json(message)
            }
            RestApiResponseError::SawtoothConnectionError(ref message) => {
                HttpResponse::ServiceUnavailable().json(message)
            }
            RestApiResponseError::DatabaseError(ref message) => {
                HttpResponse::ServiceUnavailable().json(message)
            }
            RestApiResponseError::NotFoundError(ref message) => {
                HttpResponse::NotFound().json(message)
            }
            _ => HttpResponse::InternalServerError().json("Internal Server Error"),
        }
    }
}

impl From<PayloadError> for RestApiResponseError {
    fn from(err: PayloadError) -> RestApiResponseError {
        RestApiResponseError::BadRequest(format!(
            "Payload was not well formated. {}",
            err.to_string()
        ))
    }
}

impl From<MailboxError> for RestApiResponseError {
    fn from(err: MailboxError) -> RestApiResponseError {
        RestApiResponseError::RequestHandlerError(format!(
            "Failed to deliver message to request handler. {}",
            err.to_string()
        ))
    }
}

impl From<UrlGenerationError> for RestApiResponseError {
    fn from(err: UrlGenerationError) -> RestApiResponseError {
        RestApiResponseError::RequestHandlerError(format!(
            "Failed generate response URL. {}",
            err.to_string()
        ))
    }
}

impl From<DatabaseError> for RestApiResponseError {
    fn from(err: DatabaseError) -> RestApiResponseError {
        RestApiResponseError::DatabaseError(format!(
            "Database Error occured: {}", 
            err.to_string()
        ))
    }
}

impl From<diesel::result::Error> for RestApiResponseError {
    fn from(err: diesel::result::Error) -> Self {
        RestApiResponseError::DatabaseError(format!(
            "Database Result Error occured: {}",
            err.to_string()
        ))
    }
}
/*
impl From<io::Error> for RestApiResponseError {
    fn from(err: io::Error) -> Self {
        RestApiResponseError::IoError(err)
    }
}

impl From<protobuf::ProtobufError> for RestApiResponseError {
    fn from(err: protobuf::ProtobufError) -> Self {
        RestApiResponseError::ProtobufError(err)
    }
}

impl From<signing::Error> for RestApiResponseError {
    fn from(err: signing::Error) -> Self {
        RestApiResponseError::SigningError(err)
    }
}

impl From<SendError> for RestApiResponseError {
    fn from(err: SendError) -> Self {
        RestApiResponseError::SendError(err)
    }
}
*/
impl From<protos::ProtoConversionError> for RestApiResponseError {
    fn from(err: protos::ProtoConversionError) -> Self {
        RestApiResponseError::GridProtoError(err)
    }
}
/*
impl From<sabre_sdk::protos::ProtoConversionError> for RestApiResponseError {
    fn from(err: sabre_sdk::protos::ProtoConversionError) -> Self {
        RestApiResponseError::SabreProtoError(err)
    }
}
*/
//use grid_sdk::protos;
//use sawtooth_sdk::signing;
use std::error::Error as StdError;
//use std::io;

#[derive(Debug)]
pub enum CliError {
    LoggingInitializationError(Box<flexi_logger::FlexiLoggerError>),
    InvalidYamlError(String),
    PayloadError(String),
    UserError(String),
    DatabaseError(String),
    SigningError(signing::Error),
    IoError(io::Error),
    ProtobufError(protobuf::ProtobufError),
    ReqwestError(reqwest::Error),
    GridProtoError(protos::ProtoConversionError),
    SabreProtoError(sabre_sdk::protos::ProtoConversionError),
}

impl StdError for CliError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            CliError::LoggingInitializationError(err) => Some(err),
            CliError::InvalidYamlError(_) => None,
            CliError::PayloadError(_) => None,
            CliError::UserError(_) => None,
            CliError::DatabaseError(_) => None,
            CliError::IoError(err) => Some(err),
            CliError::ProtobufError(err) => Some(err),
            CliError::SigningError(err) => Some(err),
            CliError::ReqwestError(err) => Some(err),
            CliError::GridProtoError(err) => Some(err),
            CliError::SabreProtoError(err) => Some(err),
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            CliError::UserError(ref err) => write!(f, "Error: {}", err),
            CliError::InvalidYamlError(ref err) => write!(f, "InvalidYamlError: {}", err),
            CliError::PayloadError(ref err) => write!(f, "PayloadError: {}", err),
            CliError::IoError(ref err) => write!(f, "IoError: {}", err),
            CliError::DatabaseError(ref err) => write!(f, "DatabaseError: {}", err),
            CliError::SigningError(ref err) => write!(f, "SigningError: {}", err),
            CliError::ProtobufError(ref err) => write!(f, "ProtobufError: {}", err),
            CliError::LoggingInitializationError(ref err) => {
                write!(f, "LoggingInitializationError: {}", err)
            }
            CliError::ReqwestError(ref err) => write!(f, "Reqwest Error: {}", err),
            CliError::GridProtoError(ref err) => write!(f, "Grid Proto Error: {}", err),
            CliError::SabreProtoError(ref err) => write!(f, "Sabre Proto Error: {}", err),
        }
    }
}

impl From<flexi_logger::FlexiLoggerError> for CliError {
    fn from(err: flexi_logger::FlexiLoggerError) -> Self {
        CliError::LoggingInitializationError(Box::new(err))
    }
}

impl From<signing::Error> for CliError {
    fn from(err: signing::Error) -> Self {
        CliError::SigningError(err)
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        CliError::IoError(err)
    }
}

impl From<serde_yaml::Error> for CliError {
    fn from(err: serde_yaml::Error) -> Self {
        CliError::InvalidYamlError(err.to_string())
    }
}

impl From<protobuf::ProtobufError> for CliError {
    fn from(err: protobuf::ProtobufError) -> Self {
        CliError::ProtobufError(err)
    }
}

impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> Self {
        CliError::ReqwestError(err)
    }
}

impl From<protos::ProtoConversionError> for CliError {
    fn from(err: protos::ProtoConversionError) -> Self {
        CliError::GridProtoError(err)
    }
}

impl From<sabre_sdk::protos::ProtoConversionError> for CliError {
    fn from(err: sabre_sdk::protos::ProtoConversionError) -> Self {
        CliError::SabreProtoError(err)
    }
}
