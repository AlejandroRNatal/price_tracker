use std::fmt::Formatter;

#[derive(Debug)]
pub enum Error {
    MissingArgument{ arg: String },
    MissingSetMapping { set: String },
    FailedOpeningFile,
    FailedParsingFile,
    ApiKeyNotFound,
    InvalidArgument {
        arg: String,
    },
    InvalidEndpoint {
        url: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}