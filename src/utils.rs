use crate::{codec::Serialize, types::core::Tag};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::iter;

pub(crate) fn gen_tag() -> Tag {
    let mut rng = thread_rng();
    Tag(iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(8)
        .collect())
}

pub(crate) fn join<T: std::fmt::Display>(elements: &[T], sep: &str) -> String {
    elements
        .iter()
        .map(|x| format!("{}", x))
        .collect::<Vec<String>>()
        .join(sep)
}

pub(crate) fn join_bytes(elements: Vec<Vec<u8>>, sep: &[u8]) -> Vec<u8> {
    elements
        .iter()
        .map(|x| x.to_vec())
        .collect::<Vec<Vec<u8>>>()
        .join(sep)
}

pub(crate) fn join_serializable<I: Serialize>(elements: &[I], sep: &[u8]) -> Vec<u8> {
    elements
        .iter()
        .map(|x| x.serialize())
        .collect::<Vec<Vec<u8>>>()
        .join(sep)
}
