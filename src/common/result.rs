use std::any::Any;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, HttpResponseBuilder};
use serde::Serialize;
use std::cell::BorrowMutError;
use std::fmt::{Debug, Display, Formatter};
use std::sync::PoisonError;
use log::error;
use serde::ser::StdError;

pub enum Error {
    Success,

    // region inner error

    Common(String),
    Busy(String),
    CastDbValueError(String),
    ConnectError(String),
    CastToJsonError,
    ParseJsonError(String),
    DirectoryError(String),
    SqliteError(rusqlite::Error),
    ConcurrentRequests,
    TimestampError(i64),
    NoSuchFile,

    // endregion

    // region repo

    NoSuchRepo,
    UsedRepoName,
    SomeConfigCanNotChange,

    // endregion

    // region user error

    NeedLogin,
    NoSuchUser,
    NickOrPasswordError,
    UsedNick,
    PermissionCheckFailed,

    // endregion

    // region item error

    ItemNotFound,
    UnknownFileContentType,
    ImageLoadError(String),

    // endregion

    // region tag error

    TagNotFound,
    TagRelationNotFound,

    // endregion
}

pub type Res<T, E = Error> = Result<T, E>;

impl Error {
    pub fn msg(&self) -> String {
        match self {
            Error::Success => String::from("Success"),

            Error::Common(str) => String::from(str),
            Error::Busy(str) => String::from(str),
            Error::CastDbValueError(str) => String::from(str),
            Error::ConnectError(str) => String::from(str),
            Error::CastToJsonError => String::from("cast to json fail"),
            Error::ParseJsonError(str) => String::from(str),
            Error::DirectoryError(str) => String::from(str),
            Error::SqliteError(err) => String::from(err.to_string()),

            Error::NeedLogin => String::from("you are not login"),
            Error::NoSuchUser => String::from("no such user"),
            Error::NickOrPasswordError => String::from("nick or password error"),
            Error::UsedNick => String::from("nick has been used"),

            Error::ItemNotFound => String::from("item not found"),
            Error::UnknownFileContentType => String::from("unknown file content-type"),
            Error::ImageLoadError(str) => String::from(str),

            Error::ConcurrentRequests => String::from("busy, please retry after a minute"),
            Error::TimestampError(timestamp) => format!("timestamp from {} failed", timestamp), 
            Error::NoSuchFile => String::from("no such file"),
            Error::NoSuchRepo => String::from("no such repo"),
            Error::UsedRepoName => String::from("used repo name"),
            Error::SomeConfigCanNotChange => String::from("some repo config can not be changed"),
            Error::PermissionCheckFailed => String::from("no permission"),
            Error::TagNotFound => String::from("tag not found"),
            Error::TagRelationNotFound => String::from("tag relation not found"),
            // _ => panic!("{:?}", self)
        }
    }

    pub fn code(&self) -> StatusCode {
        match self {
            Error::Success => StatusCode::OK,

            Error::Common(_) |
            Error::Busy(_) |
            Error::CastDbValueError(_) |
            Error::ConnectError(_) |
            Error::CastToJsonError |
            Error::ParseJsonError(_) |
            Error::DirectoryError(_) |
            Error::SqliteError(_) |
            Error::TimestampError(_) |
            Error::ConcurrentRequests
            => StatusCode::BAD_GATEWAY,

            Error::NeedLogin 
            => StatusCode::UNAUTHORIZED,

            Error::PermissionCheckFailed
            => StatusCode::FORBIDDEN,

            Error::NoSuchUser |
            Error::NickOrPasswordError |
            Error::UsedNick |
            Error::ItemNotFound |
            Error::UnknownFileContentType |
            Error::NoSuchFile |
            Error::NoSuchRepo |
            Error::UsedRepoName |
            Error::TagNotFound |
            Error::SomeConfigCanNotChange |
            Error::TagRelationNotFound |
            Error::ImageLoadError(_)
            => StatusCode::BAD_REQUEST,

            // _ => panic!("{:?}", self)
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(&self.msg())?;
        Ok(())
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg())?;
        Ok(())
    }
}

impl<Guard> From<PoisonError<Guard>> for Error {
    fn from(value: PoisonError<Guard>) -> Self {
        Error::Busy(value.to_string())
    }
}

impl From<BorrowMutError> for Error {
    fn from(value: BorrowMutError) -> Self {
        Error::Busy(value.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::DirectoryError(value.to_string())
    }
}

impl StdError for Error {}

impl Error {
    pub fn to_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.code()).body(self.msg())
    }
}

pub fn to_response<T: Serialize>(result: Res<T>) -> HttpResponse {
    match result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => HttpResponseBuilder::new(err.code()).body(err.msg())
    }
}