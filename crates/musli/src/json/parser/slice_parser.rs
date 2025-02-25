use crate::json::error::ErrorMessage;
use crate::json::parser::{Parser, StringReference, Token};
use crate::{Buf, Context};

/// An efficient [`Parser`] wrapper around a slice.
pub(crate) struct SliceParser<'de> {
    pub(crate) slice: &'de [u8],
    pub(crate) index: usize,
}

impl<'de> SliceParser<'de> {
    /// Construct a new instance around the specified slice.
    #[inline]
    pub(crate) fn new(slice: &'de [u8]) -> Self {
        Self { slice, index: 0 }
    }
}

impl<'de> Parser<'de> for SliceParser<'de> {
    type Mut<'this> = &'this mut SliceParser<'de> where Self: 'this;

    #[inline]
    fn borrow_mut(&mut self) -> Self::Mut<'_> {
        self
    }

    #[inline]
    fn parse_string<'scratch, C, S>(
        &mut self,
        cx: &C,
        validate: bool,
        scratch: &'scratch mut S,
    ) -> Result<StringReference<'de, 'scratch>, C::Error>
    where
        C: ?Sized + Context,
        S: ?Sized + Buf,
    {
        let start = cx.mark();
        let actual = self.peek(cx)?;

        if !matches!(actual, Token::String) {
            return Err(cx.marked_message(start, format_args!("Expected string, found {actual}")));
        }

        self.skip(cx, 1)?;
        let out = crate::json::parser::string::parse_string_slice_reader(
            cx, self, validate, start, scratch,
        );
        out
    }

    #[inline]
    fn read_byte<C>(&mut self, cx: &C) -> Result<u8, C::Error>
    where
        C: ?Sized + Context,
    {
        let mut byte = [0];
        self.read(cx, &mut byte[..])?;
        Ok(byte[0])
    }

    #[inline]
    fn skip<C>(&mut self, cx: &C, n: usize) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        let outcome = self.index.wrapping_add(n);

        if outcome > self.slice.len() || outcome < self.index {
            return Err(cx.message("Buffer underflow"));
        }

        self.index = outcome;
        cx.advance(n);
        Ok(())
    }

    #[inline]
    fn read<C>(&mut self, cx: &C, buf: &mut [u8]) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        let outcome = self.index.wrapping_add(buf.len());

        if outcome > self.slice.len() || outcome < self.index {
            return Err(cx.message("Buffer underflow"));
        }

        buf.copy_from_slice(&self.slice[self.index..outcome]);
        self.index = outcome;
        cx.advance(buf.len());
        Ok(())
    }

    #[inline]
    fn skip_whitespace<C>(&mut self, cx: &C) -> Result<(), C::Error>
    where
        C: ?Sized + Context,
    {
        while matches!(
            self.slice.get(self.index),
            Some(b' ' | b'\n' | b'\t' | b'\r')
        ) {
            self.index = self.index.wrapping_add(1);
            cx.advance(1);
        }

        Ok(())
    }

    #[inline]
    fn pos(&self) -> u32 {
        self.index as u32
    }

    #[inline]
    fn peek_byte<C>(&mut self, _: &C) -> Result<Option<u8>, C::Error>
    where
        C: ?Sized + Context,
    {
        Ok(self.slice.get(self.index).copied())
    }

    fn parse_f32<C>(&mut self, cx: &C) -> Result<f32, C::Error>
    where
        C: ?Sized + Context,
    {
        let Some((value, read)) = crate::dec2flt::dec2flt(&self.slice[self.index..]) else {
            return Err(cx.custom(ErrorMessage::ParseFloat));
        };

        self.index += read;
        cx.advance(read);
        Ok(value)
    }

    fn parse_f64<C>(&mut self, cx: &C) -> Result<f64, C::Error>
    where
        C: ?Sized + Context,
    {
        let Some((value, read)) = crate::dec2flt::dec2flt(&self.slice[self.index..]) else {
            return Err(cx.custom(ErrorMessage::ParseFloat));
        };

        self.index += read;
        cx.advance(read);
        Ok(value)
    }
}
