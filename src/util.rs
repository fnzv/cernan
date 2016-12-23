use hopper;
use metric;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops;
use std::sync;

impl CString {
    pub fn set(&mut self, s: &str) {
        let inner = sync::Arc::make_mut(&mut self.inner);
        inner.clear();
        inner.push_str(s);
    }
}

impl<'a> From<&'a str> for CString {
    fn from(s: &'a str) -> CString {
        CString { inner: sync::Arc::new(s.to_string()) }
    }
}

impl<'a> From<sync::Arc<String>> for CString {
    fn from(s: sync::Arc<String>) -> CString {
        CString { inner: s.clone() }
    }
}

impl AsRef<str> for CString {
    #[inline]
    fn as_ref(&self) -> &str {
        &(*self.inner)
    }
}

impl fmt::Display for CString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl PartialOrd for CString {
    fn partial_cmp(&self, other: &CString) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CString {
    fn cmp(&self, other: &CString) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl ops::Index<ops::RangeFull> for CString {
    type Output = str;

    #[inline]
    fn index(&self, _index: ops::RangeFull) -> &str {
        &self.inner[..]
    }
}

macro_rules! impl_eq {
    ($lhs:ty, $rhs: ty) => {
         impl<'a, 'b> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool { PartialEq::eq(&self[..], &other[..]) }
            #[inline]
            fn ne(&self, other: &$rhs) -> bool { PartialEq::ne(&self[..], &other[..]) }
        }

         impl<'a, 'b> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool { PartialEq::eq(&self[..], &other[..]) }
            #[inline]
            fn ne(&self, other: &$lhs) -> bool { PartialEq::ne(&self[..], &other[..]) }
        }

    }
}

impl_eq! { CString, str }
impl_eq! { CString, &'a str }

impl PartialEq for CString {
    fn eq(&self, other: &CString) -> bool {
        self.inner == other.inner
    }
}
impl Eq for CString {}

impl Hash for CString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

include!(concat!(env!("OUT_DIR"), "/util_types.rs"));

pub type Channel = Vec<hopper::Sender<metric::Event>>;

#[inline]
pub fn send<S>(_ctx: S, chans: &mut Channel, event: metric::Event)
    where S: Into<String> + fmt::Display
{
    let max = chans.len() - 1;
    if max == 0 {
        chans[0].send(event)
    } else {
        for mut chan in &mut chans[1..] {
            chan.send(event.clone());
        }
        chans[0].send(event);
    }
}
