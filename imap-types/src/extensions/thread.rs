use std::{
    fmt::{Display, Formatter},
    num::NonZeroU32,
};

#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::core::{Atom, Vec1, Vec2};

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Thread {
    Members {
        prefix: Vec1<NonZeroU32>,
        answers: Option<Vec2<Thread>>,
    },
    Nested {
        answers: Vec2<Thread>,
    },
}

impl Display for Thread {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let empty_answers: Vec<Thread> = vec![];

        write!(f, "(")?;
        let mut stack = match self {
            Self::Members { prefix, answers } => {
                write_prefix(f, prefix)?;
                match answers {
                    Some(answers) => {
                        write!(f, " ")?;
                        vec![answers.as_ref().iter()]
                    }
                    None => vec![empty_answers.iter()],
                }
            }
            Self::Nested { answers } => {
                vec![answers.as_ref().iter()]
            }
        };

        loop {
            if let Some(answers) = stack.last_mut() {
                if let Some(thread) = answers.next() {
                    let thing = match thread {
                        Self::Members { prefix, answers } => {
                            write!(f, "(")?;
                            write_prefix(f, prefix)?;
                            match answers {
                                Some(answers) => {
                                    write!(f, " ")?;
                                    answers.as_ref().iter()
                                }
                                None => empty_answers.iter(),
                            }
                        }
                        Self::Nested { answers } => {
                            write!(f, "(")?;
                            answers.as_ref().iter()
                        }
                    };

                    stack.push(thing);
                } else {
                    stack.pop();
                    write!(f, ")")?;
                }
            } else {
                break;
            }
        }

        Ok(())
    }
}

fn write_prefix(f: &mut Formatter, prefix: &Vec1<NonZeroU32>) -> std::fmt::Result {
    let (head, tail) = prefix.as_ref().split_first().unwrap();

    write!(f, "{}", head)?;
    for element in tail {
        write!(f, " {}", element)?;
    }

    Ok(())
}

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ThreadingAlgorithm<'a> {
    OrderedSubject,
    References,
    Other(ThreadingAlgorithmOther<'a>),
}

impl<'a> From<Atom<'a>> for ThreadingAlgorithm<'a> {
    fn from(value: Atom<'a>) -> Self {
        match value.as_ref().to_lowercase().as_ref() {
            "orderedsubject" => Self::OrderedSubject,
            "references" => Self::References,
            _ => Self::Other(ThreadingAlgorithmOther(value)),
        }
    }
}

impl Display for ThreadingAlgorithm<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(match self {
            ThreadingAlgorithm::OrderedSubject => "ORDEREDSUBJECT",
            ThreadingAlgorithm::References => "REFERENCES",
            ThreadingAlgorithm::Other(other) => other.as_ref(),
        })
    }
}

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThreadingAlgorithmOther<'a>(Atom<'a>);

impl<'a> AsRef<str> for ThreadingAlgorithmOther<'a> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
