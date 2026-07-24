/*
 * Radioware Source
 * Copyright (C) 2026 Luca Cireddu <sardylan@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see <https://www.gnu.org/licenses/>.
 *
 */

use std::any::Any;
use std::fmt::{Display, Formatter};
use std::sync::mpsc::{RecvError, SendError};

#[derive(Debug)]
pub(crate) enum Error {
    Configuration(String),
    IO(std::io::Error),
    Channel(String),
    Codec(String),
    System,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Configuration(e) => write!(f, "Configuration error: {}", e),
            Error::IO(e) => write!(f, "IO error: {}", e),
            Error::Channel(e) => write!(f, "Channel error: {}", e),
            Error::Codec(e) => write!(f, "Codec error: {}", e),
            Error::System => write!(f, "System error"),
        }
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IO(value)
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(value: SendError<T>) -> Self {
        Error::Channel(value.to_string())
    }
}

impl From<RecvError> for Error {
    fn from(value: RecvError) -> Self {
        Error::Channel(value.to_string())
    }
}

impl From<Box<dyn Any + Send + 'static>> for Error {
    fn from(_: Box<dyn Any + Send + 'static>) -> Self {
        Error::System
    }
}
