use std::{error, fmt};

#[derive(Debug, PartialEq)]
pub enum Error {
    PathProblem,
    FailedStdoutCapture,
    MissingArg(String)
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::PathProblem => write!(f, "Dir does not exist or insufficient permissions."),
            Error::FailedStdoutCapture => write!(f, "Failed to capture parent's stdout."),
            Error::MissingArg(s) => write!(f, "Missing -{} argument.", s)
        }
    }
}

