use codec::Serialize;
use std::io::Write;

pub mod codec;
pub mod parse;
pub mod state;
pub mod types;
pub mod utils;

struct List1OrNil<'a, T>(&'a Vec<T>, &'a [u8]);

impl<'a, T> Serialize for List1OrNil<'a, T>
where
    T: Serialize,
{
    fn serialize(&self, writer: &mut impl Write) -> std::io::Result<()> {
        if let Some((last, head)) = self.0.split_last() {
            writer.write_all(b"(")?;

            for item in head {
                item.serialize(writer)?;
                writer.write_all(self.1)?;
            }

            last.serialize(writer)?;

            writer.write_all(b")")
        } else {
            writer.write_all(b"NIL")
        }
    }
}

struct List1AttributeValueOrNil<'a, T>(&'a Vec<(T, T)>);

impl<'a, T> Serialize for List1AttributeValueOrNil<'a, T>
where
    T: Serialize,
{
    fn serialize(&self, writer: &mut impl Write) -> std::io::Result<()> {
        if let Some((last, head)) = self.0.split_last() {
            writer.write_all(b"(")?;

            for (attribute, value) in head {
                attribute.serialize(writer)?;
                writer.write_all(b" ")?;
                value.serialize(writer)?;
                writer.write_all(b" ")?;
            }

            let (attribute, value) = last;
            attribute.serialize(writer)?;
            writer.write_all(b" ")?;
            value.serialize(writer)?;

            writer.write_all(b")")
        } else {
            writer.write_all(b"NIL")
        }
    }
}
