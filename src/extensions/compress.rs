//! The IMAP COMPRESS Extension

// Additional changes:
//
// command-auth   =/ compress
// capability     =/ "COMPRESS=" algorithm
// resp-text-code =/ "COMPRESSIONACTIVE"

use imap_types::{command::CommandBody, message::CompressionAlgorithm};
use nom::{
    bytes::streaming::tag_no_case,
    combinator::{map, value},
    sequence::preceded,
    IResult,
};

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

#[cfg(test)]
mod tests {
    use imap_types::{command::CommandBody, message::CompressionAlgorithm};

    use super::*;

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
