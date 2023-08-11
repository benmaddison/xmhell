use core::fmt;

use quick_xml::events::Event;

/// Library error types.
#[derive(Debug)]
pub enum Error {
    /// An error occurred while reading from a [`quick_xml::Reader`].
    Reader(quick_xml::Error),
    /// An unexpected [`quick_xml::events::Event`] was encountered while
    /// reading.
    UnexpectedEvent(Event<'static>),
    /// An error was returned while processing the inner content of an XML node.
    Inner(Box<dyn std::error::Error + Send + Sync + 'static>),
    /// End-of-file while reading input.
    Eof,
}

impl Error {
    pub(crate) fn unexpected_event(event: Event<'_>) -> Self {
        Self::UnexpectedEvent(event.into_owned())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reader(err) => write!(f, "XML read error: {err}"),
            Self::UnexpectedEvent(event) => write!(f, "unexpected XML event: {event:?}"),
            Self::Inner(err) => write!(f, "Error while reading inner content: {err}"),
            Self::Eof => write!(f, "End-of-file while reading inner content"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Reader(err) => Some(err),
            Self::Inner(err) => Some(err.as_ref()),
            Self::UnexpectedEvent(_) | Self::Eof => None,
        }
    }
}

impl From<quick_xml::Error> for Error {
    fn from(err: quick_xml::Error) -> Self {
        Self::Reader(err)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for Error {
    fn from(err: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        Self::Inner(err)
    }
}
