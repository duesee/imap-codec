//! The IMAP COMPRESS Extension

// Additional changes:
//
// command-auth   =/ compress
// capability     =/ "COMPRESS=" algorithm
// resp-text-code =/ "COMPRESSIONACTIVE"

use std::io::Write;

use imap_types::{command::CommandBody, message::CompressionAlgorithm};
use nom::{
    bytes::streaming::tag_no_case,
    combinator::{map, value},
    sequence::preceded,
    IResult,
};

use crate::codec::Encode;

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

impl Encode for CompressionAlgorithm {
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            CompressionAlgorithm::Deflate => writer.write_all(b"DEFLATE"),
        }
    }
}

#[cfg(test)]
mod tests {
    use imap_types::{command::CommandBody, message::CompressionAlgorithm};

    use super::*;
    use crate::testing::known_answer_test_encode;

    #[test]
    fn test_encode_command_body_compress() {
        let tests = [(
            CommandBody::compress(CompressionAlgorithm::Deflate),
            b"COMPRESS DEFLATE".as_ref(),
        )];

        for test in tests {
            known_answer_test_encode(test);
        }
    }

    #[test]
    fn test_parse_compress() {
        let tests = [
            (
                b"compress deflate ".as_ref(),
                Ok((
                    b" ".as_ref(),
                    CommandBody::compress(CompressionAlgorithm::Deflate),
                )),
            ),
            (b"compress deflat ".as_ref(), Err(())),
            (b"compres deflate ".as_ref(), Err(())),
            (b"compress  deflate ".as_ref(), Err(())),
        ];

        for (test, expected) in tests {
            match expected {
                Ok((expected_rem, expected_object)) => {
                    let (got_rem, got_object) = compress(test).unwrap();
                    assert_eq!(expected_object, got_object);
                    assert_eq!(expected_rem, got_rem);
                }
                Err(_) => {
                    assert!(compress(test).is_err())
                }
            }
        }
    }
}
