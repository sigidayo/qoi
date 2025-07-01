#[derive(Debug)]
pub enum DecodeError {
    InvalidHeader(HeaderError),
}

#[derive(Debug)]
pub enum HeaderError {
    MalformedInput(core::array::TryFromSliceError),
    InvalidMagicBytes {
        expected: &'static str,
        found: String,
    },
    InvalidColourChannels {
        expected: &'static str,
        found: String,
    },
    InvalidColourSpace {
        expected: &'static str,
        found: String,
    },
}

impl From<HeaderError> for DecodeError {
    fn from(e: HeaderError) -> DecodeError {
        DecodeError::InvalidHeader(e)
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DecodeError::InvalidHeader(e) => write!(f, "invalid header: {}", e),
        }
    }
}
#[cfg(feature = "std")]
impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DecodeError::InvalidHeader(e) => Some(e),
        }
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for HeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HeaderError::MalformedInput(e) => write!(f, "malformed input: {e}"),
            HeaderError::InvalidMagicBytes { expected, found } => write!(f, "invalid magic bytes (expected {expected:?}, found {found:?})"),
            HeaderError::InvalidColourChannels { expected, found } => write!(f, "invalid colour channels (expected {expected:?}, found {found:?})"),
            HeaderError::InvalidColourSpace { expected, found } => write!(f, "invalid colour space (expected {expected:?}, found {found:?})"),
        }
    }
}
#[cfg(feature = "std")]
impl std::error::Error for HeaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HeaderError::MalformedInput(e) => Some(e),
            HeaderError::InvalidMagicBytes { .. } => None,
            HeaderError::InvalidColourChannels { .. } => None,
            HeaderError::InvalidColourSpace { .. } => None,
        }
    }
}