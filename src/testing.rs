use std::fmt::Debug;

use nom::IResult;

use crate::{
    codec::{Decode, Encode},
    command::Command,
    response::{Continue, Greeting, Response},
    utils::escape_byte_string,
};

pub fn known_answer_test_encode((test_object, expected_bytes): (impl Encode, impl AsRef<[u8]>)) {
    let expected_bytes = expected_bytes.as_ref();
    let got_bytes = test_object.encode_detached().unwrap();
    let got_bytes = got_bytes.as_slice();

    if expected_bytes != got_bytes {
        println!("# Debug (`escape_byte_string`, encapsulated by `<<<` and `>>>`)");
        println!(
            "Left:  <<<{}>>>\nRight: <<<{}>>>",
            escape_byte_string(expected_bytes),
            escape_byte_string(got_bytes),
        );
        println!("# Debug");
        panic!("Left:  {:02x?}\nRight: {:02x?}", expected_bytes, got_bytes);
    }
}

pub fn known_answer_test_parse<'a, O, P>(
    (test, expected_remainder, expected_object): (&'a [u8], &[u8], O),
    parser: P,
) where
    O: Debug + Eq + 'a,
    P: Fn(&'a [u8]) -> IResult<&'a [u8], O>,
{
    let (got_remainder, got_object) = parser(test).unwrap();
    assert_eq!(expected_remainder, got_remainder);
    assert_eq!(expected_object, got_object);
}

// Note: Maybe there is a cleaner way to write this using generic bounds. However,
// we tried it and failed to provide a cleaner solution. Thus, it's a macro for now.
macro_rules! impl_kat_inverse {
    ($fn_name:ident, $object:ty) => {
        pub fn $fn_name(tests: &[(&[u8], &[u8], $object)]) {
            for (no, (test_input, expected_remainder, expected_object)) in tests.iter().enumerate()
            {
                println!("# {no}");

                let (got_remainder, got_object) = <$object>::decode(test_input).unwrap();
                assert_eq!(*expected_object, got_object);
                assert_eq!(*expected_remainder, got_remainder);

                let got_output = got_object.encode_detached().unwrap();

                // This second `decode` makes using generic bonuds more complicated due to the
                // different lifetime.
                let (got_remainder, got_object_again) = <$object>::decode(&got_output).unwrap();
                assert_eq!(got_object, got_object_again);
                assert!(got_remainder.is_empty());
            }
        }
    };
}

impl_kat_inverse! {kat_inverse_greeting, Greeting}
impl_kat_inverse! {kat_inverse_command, Command}
impl_kat_inverse! {kat_inverse_response, Response}
impl_kat_inverse! {kat_inverse_continue, Continue}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{Command, CommandBody};

    #[test]
    #[should_panic]
    fn test_known_answer_test_encode() {
        known_answer_test_encode((Command::new("A", CommandBody::Noop).unwrap(), b""));
    }
}
