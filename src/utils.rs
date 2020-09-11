use crate::{codec::Serialize, types::core::Tag};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::{io::Write, iter};

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

pub(crate) fn join_serializable<I: Serialize>(
    elements: &[I],
    sep: &[u8],
    writer: &mut impl Write,
) -> std::io::Result<()> {
    if let Some((last, head)) = elements.split_last() {
        for item in head {
            item.serialize(writer)?;
            writer.write_all(sep)?;
        }

        last.serialize(writer)
    } else {
        Ok(())
    }
}
