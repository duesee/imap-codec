use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::iter;

pub(crate) fn gen_tag() -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(8)
        .collect()
}

pub fn join<T: std::fmt::Display>(elements: &[T], sep: &str) -> String {
    elements
        .iter()
        .map(|x| format!("{}", x))
        .collect::<Vec<String>>()
        .join(sep)
}

pub fn join_bytes(elements: Vec<Vec<u8>>, sep: &[u8]) -> Vec<u8> {
    elements
        .iter()
        .map(|x| x.to_vec())
        .collect::<Vec<Vec<u8>>>()
        .join(sep)
}

pub fn join_or_nil<T: std::fmt::Display>(elements: &[T], sep: &str) -> String {
    if elements.is_empty() {
        String::from("nil")
    } else {
        String::from("(")
            + &elements
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join(sep)
            + ")"
    }
}
