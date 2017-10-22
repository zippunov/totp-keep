// TODO propep error messages with detalisation

#[derive(Debug)]
pub enum Error {
    NoHomeDirectory,
    FileNotFound,
    FileError,
    CorruptedFileContent,
    WrongPassword,
    WrongServiceRecordData,
    UnknownCommand
}

use std::fmt;
use std::error::Error as StdError;
use std::convert::From;
use std::io;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::NoHomeDirectory => f.write_str("NoHomeDirectory"),
            Error::FileNotFound => f.write_str("FileNotFound"),
            Error::FileError => f.write_str("FileEror"),
            Error::CorruptedFileContent => f.write_str("CorruptedFileContent"),
            Error::WrongPassword => f.write_str("WrongPassword"),
            Error::WrongServiceRecordData => f.write_str("WrongServiceRecord"),
            Error::UnknownCommand => f.write_str("UnknownCommand"),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::NoHomeDirectory => "Unable to get home directory",
            Error::FileNotFound => "File not found",
            Error::FileError => "File operation error",
            Error::CorruptedFileContent => "File is corrupted",
            Error::WrongPassword => "Wrong password",
            Error::WrongServiceRecordData => "Unable to parse TOTP secret",
            Error::UnknownCommand => "Unknown Command",
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::FileError
    }
}

