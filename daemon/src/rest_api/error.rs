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
    IoError(String),
    ProtobufError(String),
    SigningError(String),
    GridProtoError(String),
    SabreProtoError(String),
    //IoError(io::Error),
    //ProtobufError(protobuf::ProtobufError),
    //SigningError(signing::Error),
    //SendError(SendError),
    //GridProtoError(protos::ProtoConversionError),
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
            RestApiResponseError::IoError(_) => None,
            RestApiResponseError::ProtobufError(_) => None,
            RestApiResponseError::SigningError(_) => None,
            RestApiResponseError::GridProtoError(_) => None,
            RestApiResponseError::SabreProtoError(_) => None,
            //RestApiResponseError::IoError(err) => Some(err),
            //RestApiResponseError::ProtobufError(err) => Some(err),
            //RestApiResponseError::SigningError(err) => Some(err),
            //RestApiResponseError::SendError(err) => Some(err),
            //RestApiResponseError::GridProtoError(err) => Some(err),
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
            RestApiResponseError::IoError(ref err) => write!(f, "IoError: {}", err),
            RestApiResponseError::ProtobufError(ref err) => write!(f, "ProtobufError: {}", err),
            RestApiResponseError::SigningError(ref err) => write!(f, "SigningError: {}", err),
            //RestApiResponseError::SendError(ref err) => write!(f, "SendError: {}", err),
            RestApiResponseError::GridProtoError(ref err) => write!(f, "Grid Proto Error: {}", err),
            RestApiResponseError::SabreProtoError(ref err) => write!(f, "Sabre Proto Error: {}", err),
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
/*
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

impl From<sabre_sdk::protos::ProtoConversionError> for RestApiResponseError {
    fn from(err: sabre_sdk::protos::ProtoConversionError) -> Self {
        RestApiResponseError::SabreProtoError(err)
    }
}
