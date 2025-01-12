use std::num::NonZeroU64;

use nom::combinator::map_res;

use crate::{core::number64, decode::IMAPResult};

/// Positive unsigned 64-bit integer (mod-sequence) (1 <= n < 18,446,744,073,709,551,615)
///
/// ```abnf
/// mod-sequence-value  = 1*DIGIT
/// ```
#[cfg(feature = "ext_condstore_qresync")]
pub(crate) fn mod_sequence_value(input: &[u8]) -> IMAPResult<&[u8], NonZeroU64> {
    map_res(number64, NonZeroU64::try_from)(input)
}

#[cfg(test)]
mod tests {
    use crate::response::resp_text;

    #[test]
    fn test_condstore_qresync_codes() {
        assert!(resp_text(b"[MODIFIED 7,9] Conditional STORE failed\r\n").is_ok());
        assert!(resp_text(
            b"[NOMODSEQ] Sorry, this mailbox format doesn't support modsequences\r\n"
        )
        .is_ok());
        assert!(resp_text(b"[HIGHESTMODSEQ 715194045007] Highest\r\n").is_ok());
    }
}
