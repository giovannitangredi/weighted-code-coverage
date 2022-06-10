use std::sync::MutexGuard;
use std::sync::PoisonError;

use thiserror::Error;

/// Customized error messages using thiserror library
#[derive(Error, Debug)]
pub enum Error {
    #[error("Error while reading Files from project folder")]
    WrongFile(),
    #[error("Error while reading json")]
    WrongJSONFile(),
    #[error("Error while converting JSON value to a type")]
    ConversionError(),
    #[error("Error while getting value from hashmap")]
    HashMapError(),
    #[error("Failing reading JSON from string")]
    ReadingJSONError(),
    #[error("Error while computing Metrics")]
    MetricsError(),
    #[error("Error while guessing language")]
    LanguageError(),
    #[error("Error while writing on csv")]
    WritingError(),
    #[error("Error during concurrency")]
    ConcurrentError(),
    #[error("Json Type is not supported! Only coveralls and covdir are supported.")]
    TypeError(),
    #[error("Error while converting path to string")]
    PathConversionError(),
    #[error("Error while locking mutex")]
    MutexError(),
    #[error(
        "Thresholds must be only 4 in this order -t SIFIS_PLAIN, SIFIS_QUANTIZED, CRAP, SKUNK"
    )]
    ThresholdsError(),
}

impl From<std::io::Error> for Error {
    fn from(_item: std::io::Error) -> Self {
        Error::WrongFile()
    }
}

impl From<serde_json::Error> for Error {
    fn from(_item: serde_json::Error) -> Self {
        Error::WrongJSONFile()
    }
}

impl From<csv::Error> for Error {
    fn from(_item: csv::Error) -> Self {
        Error::WritingError()
    }
}

impl<T> From<PoisonError<MutexGuard<'_, T>>> for Error {
    fn from(_item: PoisonError<MutexGuard<'_, T>>) -> Self {
        Error::MutexError()
    }
}

impl From<Box<dyn std::any::Any + Send>> for Error {
    fn from(_item: Box<dyn std::any::Any + Send>) -> Self {
        Error::ConcurrentError()
    }
}
