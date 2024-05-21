#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use imap_codec::fuzz::fuzz_tag_imap;
use libfuzzer_sys::fuzz_target;

/// Define a struct that represents fuzz target inputs.
/// Includes both potentially valid and invalid tag patterns.
#[derive(Debug)]
struct TagInput {
    /// String representing a valid or invalid IMAP tag
    tag: String,
    /// Additional field to simulate complex scenarios or invalid patterns
    noise: Option<String>,
}

impl<'a> Arbitrary<'a> for TagInput {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let tag = u.arbitrary::<String>()?;
        let noise = u.arbitrary::<Option<String>>()?;
        Ok(Self { tag, noise })
    }
}

fuzz_target!(|data: TagInput| {
    // Combine tag and noise to generate the input
    let mut input = data.tag;
    if let Some(noise) = data.noise {
        input.push_str(&noise);
    }

    // Convert the input string to bytes
    let input_bytes = input.as_bytes();

    // Attempt to parse the input as an IMAP tag
    match fuzz_tag_imap(input_bytes) {
        Ok((_, parsed_tag)) => {
            // If parsing succeeds, validate the output
            // Ensure the parsed tag is a subset of the original input
            assert!(input.starts_with(parsed_tag.as_ref()));
            // Optionally, check if the remaining bytes match the expected pattern for noise
            // add more specific validations based on fuzzing requirements
        }
        Err(_) => {
            // If parsing fails, check specific conditions
            // Ensure that failure is due to the expected reasons
            // Check for specific patterns that are known to be invalid
        }
    }
    // Additional checks for performance metrics or memory safety checks
});
