//! Raw nom parsers for the formal syntax of IMAP ([RFC3501](https://datatracker.ietf.org/doc/html/rfc3501#section-9)) and IMAP extensions.

/// IMAP4 IDLE command
#[cfg(feature = "ext_idle")]
pub mod rfc2177 {
    #[doc(inline)]
    pub use crate::extensions::rfc2177::*;
}

/// INTERNET MESSAGE ACCESS PROTOCOL - VERSION 4rev1
pub mod rfc3501 {
    #[doc(inline)]
    pub use crate::rfc3501::address::*;
    #[doc(inline)]
    pub use crate::rfc3501::body::*;
    #[doc(inline)]
    pub use crate::rfc3501::command::*;
    #[doc(inline)]
    pub use crate::rfc3501::core::*;
    #[doc(inline)]
    pub use crate::rfc3501::datetime::*;
    #[doc(inline)]
    pub use crate::rfc3501::envelope::*;
    #[doc(inline)]
    pub use crate::rfc3501::fetch_attributes::*;
    #[doc(inline)]
    pub use crate::rfc3501::flag::*;
    #[doc(inline)]
    pub use crate::rfc3501::mailbox::*;
    #[doc(inline)]
    pub use crate::rfc3501::response::*;
    #[doc(inline)]
    pub use crate::rfc3501::section::*;
    #[doc(inline)]
    pub use crate::rfc3501::sequence::*;
    #[doc(inline)]
    pub use crate::rfc3501::status_attributes::*;
    #[doc(inline)]
    pub use crate::rfc3501::*;
}

/// The IMAP COMPRESS Extension
#[cfg(feature = "ext_compress")]
pub mod rfc4978 {
    #[doc(inline)]
    pub use crate::extensions::rfc4987::*;
}

/// The IMAP ENABLE Extension
#[cfg(feature = "ext_enable")]
pub mod rfc5161 {
    #[doc(inline)]
    pub use crate::extensions::rfc5161::*;
}
