/// IMAP4 IDLE command
#[cfg(feature = "ext_idle")]
pub mod rfc2177 {
    #[doc(inline)]
    pub use crate::extensions::rfc2177::parse::*;
}

/// INTERNET MESSAGE ACCESS PROTOCOL - VERSION 4rev1
pub mod rfc3501 {
    #[doc(inline)]
    pub use crate::parse::address::*;
    #[doc(inline)]
    pub use crate::parse::body::*;
    #[doc(inline)]
    pub use crate::parse::command::*;
    #[doc(inline)]
    pub use crate::parse::core::*;
    #[doc(inline)]
    pub use crate::parse::datetime::*;
    #[doc(inline)]
    pub use crate::parse::envelope::*;
    #[doc(inline)]
    pub use crate::parse::fetch_attributes::*;
    #[doc(inline)]
    pub use crate::parse::flag::*;
    #[doc(inline)]
    pub use crate::parse::mailbox::*;
    #[doc(inline)]
    pub use crate::parse::response::*;
    #[doc(inline)]
    pub use crate::parse::section::*;
    #[doc(inline)]
    pub use crate::parse::sequence::*;
    #[doc(inline)]
    pub use crate::parse::status_attributes::*;
    #[doc(inline)]
    pub use crate::parse::*;
}

/// The IMAP COMPRESS Extension
#[cfg(feature = "ext_compress")]
pub mod rfc4978 {
    #[doc(inline)]
    pub use crate::extensions::rfc4987::parse::*;
}

/// The IMAP ENABLE Extension
#[cfg(feature = "ext_enable")]
pub mod rfc5161 {
    #[doc(inline)]
    pub use crate::extensions::rfc5161::parse::*;
}
