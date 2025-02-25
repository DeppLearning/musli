use core::fmt::{self, Write};
use core::ops::Deref;
use core::str;

use crate::fixed::CapacityError;
use crate::{Buf, Context};

/// Wrapper around a [`Buf`], guaranteed to be a valid utf-8 string.
pub struct BufString<B> {
    buf: B,
}

/// Collect a string into a string buffer.
pub(crate) fn collect_string<C, T>(cx: &C, value: T) -> Result<BufString<C::Buf<'_>>, C::Error>
where
    C: ?Sized + Context,
    T: fmt::Display,
{
    let Some(buf) = cx.alloc() else {
        return Err(cx.message("Failed to allocate"));
    };

    let mut string = BufString::new(buf);

    if write!(string, "{value}").is_err() {
        return Err(cx.message("Failed to write to string"));
    }

    Ok(string)
}

impl<B> BufString<B>
where
    B: Buf,
{
    /// Construct a new fixed string.
    pub(crate) const fn new(buf: B) -> BufString<B> {
        BufString { buf }
    }

    fn as_str(&self) -> &str {
        // SAFETY: Interactions ensure that data is valid utf-8.
        unsafe { str::from_utf8_unchecked(self.buf.as_slice()) }
    }

    fn try_push(&mut self, c: char) -> Result<(), CapacityError> {
        if !self.buf.write(c.encode_utf8(&mut [0; 4]).as_bytes()) {
            return Err(CapacityError);
        }

        Ok(())
    }

    fn try_push_str(&mut self, s: &str) -> Result<(), CapacityError> {
        if !self.buf.write(s.as_bytes()) {
            return Err(CapacityError);
        }

        Ok(())
    }
}

impl<B> fmt::Write for BufString<B>
where
    B: Buf,
{
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.try_push(c).map_err(|_| fmt::Error)
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.try_push_str(s).map_err(|_| fmt::Error)
    }
}

impl<B> Deref for BufString<B>
where
    B: Buf,
{
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.buf.as_slice()) }
    }
}

impl<B> fmt::Display for BufString<B>
where
    B: Buf,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<B> AsRef<str> for BufString<B>
where
    B: Buf,
{
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
