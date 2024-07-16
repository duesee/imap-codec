use std::{
    fmt::{Display, Formatter},
    num::NonZeroU32,
};

#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Unstructured};
use bounded_static_derive::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "arbitrary")]
use crate::arbitrary::impl_arbitrary_try_from;
use crate::core::{Atom, Vec1, Vec2};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
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

        while let Some(answers) = stack.last_mut() {
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

#[cfg(feature = "arbitrary")]
impl<'a> Arbitrary<'a> for Thread {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        #[cfg(not(feature = "arbitrary_simplified"))]
        return arbitrary_thread_limited(u, 7);
        #[cfg(feature = "arbitrary_simplified")]
        return arbitrary_thread_leaf(u);
    }
}

#[cfg(all(feature = "arbitrary", not(feature = "arbitrary_simplified")))]
fn arbitrary_thread_limited(u: &mut Unstructured, depth: usize) -> arbitrary::Result<Thread> {
    // We cheat a bit: Start from a leaf ...
    let mut current = arbitrary_thread_leaf(u)?;

    // ... and build up the thread to the top (with max depth == 8)
    for _ in 0..depth {
        match u.int_in_range(0..=2)? {
            0 => {
                current = Thread::Members {
                    prefix: Arbitrary::arbitrary(u)?,
                    answers: Some(Vec2::unvalidated(vec![current.clone(), current])),
                };
            }
            1 => {
                current = Thread::Nested {
                    answers: Vec2::unvalidated(vec![current.clone(), current]),
                };
            }
            2 => {
                return Ok(current);
            }
            _ => unreachable!(),
        }
    }

    Ok(current)
}

#[cfg(feature = "arbitrary")]
fn arbitrary_thread_leaf(u: &mut Unstructured) -> arbitrary::Result<Thread> {
    Ok(Thread::Members {
        prefix: Arbitrary::arbitrary(u)?,
        answers: None,
    })
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
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

#[cfg(feature = "arbitrary")]
impl_arbitrary_try_from! { ThreadingAlgorithm<'a>, Atom<'a> }

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ToStatic)]
pub struct ThreadingAlgorithmOther<'a>(Atom<'a>);

impl<'a> AsRef<str> for ThreadingAlgorithmOther<'a> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
