#![deny(missing_debug_implementations)]

#[cfg(feature = "arbitrary")]
mod arbitrary;
#[cfg(any(feature = "ext_idle", feature = "ext_enable", feature = "ext_compress"))]
pub mod extensions;
mod rfc3501;

pub use rfc3501::*;

// -- API -----------------------------------------------------------------------------------

pub mod codec;
pub mod state;
pub mod utils;

// TODO: Temporarily. Use this in tests, examples, etc. and see if it feels right.
pub mod api {
    pub mod core {
        pub use crate::rfc3501::core::{AString, Atom, AtomExt, IString, Literal, NString, Quoted};
    }

    pub mod message {
        #[cfg(feature = "ext_compress")]
        pub use crate::extensions::rfc4987::CompressionAlgorithm;
        #[cfg(feature = "ext_enable")]
        pub use crate::extensions::rfc5161::{CapabilityEnable, Utf8Kind};
        pub use crate::rfc3501::{
            core::{Charset, Tag},
            flag::{Flag, FlagNameAttribute},
            mailbox::{Mailbox, MailboxOther},
            section::{Part, Section},
            AuthMechanism, AuthMechanismOther,
        };
    }

    pub mod command {
        pub use crate::rfc3501::{
            command::{Command, CommandBody},
            mailbox::ListMailbox,
            sequence::{SeqNo, Sequence, SequenceSet, Strategy},
        };

        pub mod status {
            pub use crate::rfc3501::status_attributes::StatusAttribute;
        }

        pub mod search {
            pub use crate::rfc3501::command::SearchKey;
        }

        pub mod fetch {
            pub use crate::rfc3501::fetch_attributes::{
                FetchAttribute, Macro, MacroOrFetchAttributes,
            };
        }

        pub mod store {
            pub use crate::rfc3501::flag::{StoreResponse, StoreType};
        }
    }

    pub mod response {
        pub use crate::rfc3501::{
            core::Text,
            response::{Code, Continue, Data, Response, Status},
        };

        pub mod data {
            pub use crate::rfc3501::{
                body::{
                    BasicFields, Body, BodyStructure, MultiPartExtensionData,
                    SinglePartExtensionData, SpecificFields,
                },
                core::QuotedChar,
                envelope::Envelope,
                fetch_attributes::FetchAttributeValue,
                flag::FlagNameAttribute,
                response::{Capability, CapabilityOther},
                status_attributes::StatusAttributeValue,
            };
        }
    }
}

// -- Re-exports -----------------------------------------------------------------------------------

#[cfg(feature = "bounded-static")]
pub use bounded_static;
