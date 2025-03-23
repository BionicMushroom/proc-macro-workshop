use std::ops::{Range, RangeInclusive};

use proc_macro2::TokenStream;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Error, Ident, LitInt, Token,
};

pub struct SeqData {
    pub ident: Option<Ident>,
    pub range_kind: RangeKind,
    pub body_token_stream: TokenStream,
}

pub enum RangeKind {
    Exclusive(Range<u128>),
    Inclusive(RangeInclusive<u128>),
}

impl Parse for SeqData {
    fn parse(input: ParseStream<'_>) -> Result<Self, Error> {
        let lookahead = input.lookahead1();

        let ident = if lookahead.peek(Ident) {
            Some(input.parse::<Ident>()?)
        } else if lookahead.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            None
        } else {
            return Err(lookahead.error());
        };

        input.parse::<Token![in]>()?;

        let start = input.parse::<LitInt>()?.base10_parse::<u128>()?;
        input.parse::<Token![..]>()?;

        let lookahead = input.lookahead1();

        let range_kind = if lookahead.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            let end = input.parse::<LitInt>()?.base10_parse::<u128>()?;

            RangeKind::Inclusive(RangeInclusive::new(start, end))
        } else if lookahead.peek(LitInt) {
            let end = input.parse::<LitInt>()?.base10_parse::<u128>()?;
            RangeKind::Exclusive(Range { start, end })
        } else {
            return Err(lookahead.error());
        };

        let body;
        braced!(body in input);

        let body_token_stream = body.parse::<TokenStream>()?;

        Ok(Self {
            ident,
            range_kind,
            body_token_stream,
        })
    }
}
