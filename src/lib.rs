use codec::Codec;

pub mod codec;
pub mod parse;
pub mod state;
pub mod types;
pub mod utils;

struct List1OrNil<'a, T>(&'a Vec<T>, &'a [u8]);

impl<'a, T> Codec for List1OrNil<'a, T>
where
    T: Codec,
{
    fn serialize(&self) -> Vec<u8> {
        if let Some((last, head)) = self.0.split_last() {
            let mut out = b"(".to_vec();

            for item in head {
                out.extend(&item.serialize());
                out.extend_from_slice(self.1);
            }

            out.extend(&last.serialize());

            out.push(b')');
            out
        } else {
            b"NIL".to_vec()
        }
    }
}

struct List1AttributeValueOrNil<'a, T>(&'a Vec<(T, T)>);

impl<'a, T> Codec for List1AttributeValueOrNil<'a, T>
where
    T: Codec,
{
    fn serialize(&self) -> Vec<u8> {
        if let Some((last, head)) = self.0.split_last() {
            let mut out = b"(".to_vec();

            for (attribute, value) in head {
                out.extend(&attribute.serialize());
                out.push(b' ');
                out.extend(&value.serialize());
                out.push(b' ');
            }

            let (attribute, value) = last;
            out.extend(&attribute.serialize());
            out.push(b' ');
            out.extend(&value.serialize());

            out.push(b')');

            out
        } else {
            b"NIL".to_vec()
        }
    }
}
