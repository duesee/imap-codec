use crate::{extensions::namespace::error::NamespaceError, response::Data};

impl<'a> Data<'a> {
    pub fn namespace() -> Result<Self, NamespaceError> {
        Ok(Self::Namespace)
    }
}

/// Error-related types.
pub mod error {
    use thiserror::Error;

    /// An error that can occur when creating a `Data::Namespace` response.
    #[derive(Clone, Debug, Eq, Error, Hash, Ord, PartialEq, PartialOrd)]
    pub enum NamespaceError {}
}
