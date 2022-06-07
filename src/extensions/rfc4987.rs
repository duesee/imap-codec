//! The IMAP COMPRESS Extension

// Additional changes:
//
// command-auth   =/ compress
// capability     =/ "COMPRESS=" algorithm
// resp-text-code =/ "COMPRESSIONACTIVE"

use imap_types::{command::CommandBody, extensions::rfc4987::CompressionAlgorithm};
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
