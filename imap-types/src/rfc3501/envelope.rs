#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{core::NString, response::data::Address};

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Envelope<'a> {
    pub date: NString<'a>,
    pub subject: NString<'a>,
    pub from: Vec<Address<'a>>,
    pub sender: Vec<Address<'a>>,
    pub reply_to: Vec<Address<'a>>,
    pub to: Vec<Address<'a>>,
    pub cc: Vec<Address<'a>>,
    pub bcc: Vec<Address<'a>>,
    pub in_reply_to: NString<'a>,
    pub message_id: NString<'a>,
}
