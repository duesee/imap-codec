use std::io::Write;

use codec::Encode;

#[cfg(feature = "arbitrary")]
pub mod arbitrary;
pub mod codec;
pub mod parse;
pub mod state;
pub mod types;
pub mod utils;

struct List1OrNil<'a, T>(&'a Vec<T>, &'a [u8]);

impl<'a, T> Encode for List1OrNil<'a, T>
where
    T: Encode,
{
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        if let Some((last, head)) = self.0.split_last() {
            writer.write_all(b"(")?;

            for item in head {
                item.encode(writer)?;
                writer.write_all(self.1)?;
            }

            last.encode(writer)?;

            writer.write_all(b")")
        } else {
            writer.write_all(b"NIL")
        }
    }
}

struct List1AttributeValueOrNil<'a, T>(&'a Vec<(T, T)>);

impl<'a, T> Encode for List1AttributeValueOrNil<'a, T>
where
    T: Encode,
{
    fn encode(&self, writer: &mut impl Write) -> std::io::Result<()> {
        if let Some((last, head)) = self.0.split_last() {
            writer.write_all(b"(")?;

            for (attribute, value) in head {
                attribute.encode(writer)?;
                writer.write_all(b" ")?;
                value.encode(writer)?;
                writer.write_all(b" ")?;
            }

            let (attribute, value) = last;
            attribute.encode(writer)?;
            writer.write_all(b" ")?;
            value.encode(writer)?;

            writer.write_all(b")")
        } else {
            writer.write_all(b"NIL")
        }
    }
}
