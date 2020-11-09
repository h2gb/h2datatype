use sized_number::Context;

// Allow us to resolve either statically or dynamically, depending on what's
// needed. One or the other might throw an error, though.
#[derive(Debug, Clone, Copy)]
pub enum ResolveOffset<'a> {
    Static(u64),
    Dynamic(Context<'a>),
}

impl<'a> From<u64> for ResolveOffset<'a> {
    fn from(o: u64) -> ResolveOffset<'a> {
        ResolveOffset::Static(o)
    }
}

impl<'a> From<Context<'a>> for ResolveOffset<'a> {
    fn from(o: Context<'a>) -> ResolveOffset<'a> {
        ResolveOffset::Dynamic(o)
    }
}

impl<'a> ResolveOffset<'a> {
    pub fn position(&self) -> u64 {
        match self {
            Self::Static(n) => *n,
            Self::Dynamic(c) => c.position(),
        }
    }

    pub fn at(&self, offset: u64) -> ResolveOffset {
        match self {
            Self::Static(_) => Self::Static(offset),
            Self::Dynamic(c) => Self::Dynamic(c.at(offset)),
        }
    }
}

