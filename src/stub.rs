//! A far title. Not the page. Drop and refill
//! from home. ver is a counter home bumps.

/// The other end, enough to show, not to serve.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stub {
    title: String,
    ver: u64,
}

impl Stub {
    pub fn new(title: &str, ver: u64) -> Stub {
        Stub {
            title: title.to_string(),
            ver: ver,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn ver(&self) -> u64 {
        self.ver
    }
}
