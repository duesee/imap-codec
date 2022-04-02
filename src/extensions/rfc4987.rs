//! The IMAP COMPRESS Extension

// Additional changes:
//
// command-auth   =/ compress
// capability     =/ "COMPRESS=" algorithm
// resp-text-code =/ "COMPRESSIONACTIVE"

pub mod types {
    use std::io::Write;

    #[cfg(feature = "arbitrary")]
    use arbitrary::Arbitrary;
    #[cfg(feature = "serdex")]
    use serde::{Deserialize, Serialize};

    use crate::Encode;

    #[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
    #[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum CompressionAlgorithm {
        Deflate,
    }

    impl Encode for CompressionAlgorithm {
        fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
            match self {
                CompressionAlgorithm::Deflate => writer.write_all(b"DEFLATE"),
            }
        }
    }
}

pub(crate) mod parse {
    use nom::{
        bytes::streaming::tag_no_case,
        combinator::{map, value},
        sequence::preceded,
        IResult,
    };

    use crate::{extensions::rfc4987::types::CompressionAlgorithm, types::command::CommandBody};

    /// `algorithm = "DEFLATE"`
    pub fn algorithm(input: &[u8]) -> IResult<&[u8], CompressionAlgorithm> {
        value(CompressionAlgorithm::Deflate, tag_no_case("DEFLATE"))(input)
    }

    /// `compress = "COMPRESS" SP algorithm`
    pub fn compress(input: &[u8]) -> IResult<&[u8], CommandBody> {
        map(preceded(tag_no_case("COMPRESS "), algorithm), |algorithm| {
            CommandBody::Compress { algorithm }
        })(input)
    }
}
