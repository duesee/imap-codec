use std::fmt::Debug;

pub use imap_types::testing::known_answer_test_encode;
use nom::IResult;

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
