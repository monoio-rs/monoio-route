use crate::{error::InsertError, tree::next_param};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Segment<'a> {
    Static(&'a [u8]),
    Param(&'a [u8]),
    CatchAll(&'a [u8]),
}

impl<'a> Segment<'a> {
    #[inline(always)]
    pub const fn name(&self) -> Option<&'a [u8]> {
        match self {
            Segment::Static(_) => None,
            Segment::Param(name) => Some(name),
            Segment::CatchAll(name) => Some(name),
        }
    }
}

pub(crate) struct SegmentsIter<'a> {
    pub(crate) inner: &'a [u8],
}

impl<'a> SegmentsIter<'a> {
    #[inline(always)]
    pub(crate) const fn new(inner: &'a [u8]) -> Self {
        Self { inner }
    }
}

impl<'a> Iterator for SegmentsIter<'a> {
    type Item = Result<Segment<'a>, InsertError>;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! check_empty {
            ($var:expr) => {
                if $var.is_empty() {
                    return Some(Err(InsertError::new()));
                }
            };
        }

        macro_rules! check_special {
            ($var:expr) => {
                if memchr::memchr3(b'/', b':', b'*', $var).is_some() {
                    return Some(Err(InsertError::new()));
                }
            };
        }

        match self.inner.first()? {
            b':' => {
                let path = unsafe { self.inner.split_at_unchecked(1).1 };
                let (param_name, rest) = next_param(path);
                check_special!(param_name);
                check_empty!(param_name);
                self.inner = rest;
                Some(Ok(Segment::Param(param_name)))
            }
            b'*' => {
                let param_name = unsafe { self.inner.split_at_unchecked(1).1 };
                check_special!(param_name);
                check_empty!(param_name);
                self.inner = &[];
                Some(Ok(Segment::CatchAll(param_name)))
            }
            _ => {
                if let Some(idx) = memchr::memchr2(b':', b'*', self.inner) {
                    let (segment, rest) = unsafe { self.inner.split_at_unchecked(idx) };
                    self.inner = rest;
                    check_empty!(segment);
                    return Some(Ok(Segment::Static(segment)));
                }
                let inner = self.inner;
                self.inner = &[];
                Some(Ok(Segment::Static(inner)))
            }
        }
    }
}
