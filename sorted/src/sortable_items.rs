use proc_macro2::TokenStream;
use quote::ToTokens;
use std::{cmp::Ordering, ffi::CString, fmt::Display};
use syn::{
    Error, Ident, LitBool, LitByte, LitByteStr, LitCStr, LitChar, LitFloat, LitInt, LitStr, Path,
};

pub type StringSegments = Vec<String>;
pub type ByteString = Vec<u8>;

pub enum ValueOrWildcard<T> {
    Value(T),
    Wildcard,
}

pub struct SortableItem<T> {
    pub value_or_wildcard: ValueOrWildcard<T>,
    pub token_stream: TokenStream,
}

impl<T> SortableItem<T> {
    pub fn new_with_value(value: T, token_stream: TokenStream) -> Self {
        SortableItem {
            value_or_wildcard: ValueOrWildcard::Value(value),
            token_stream,
        }
    }

    pub fn new_with_wildcard(token_stream: TokenStream) -> Self {
        SortableItem {
            value_or_wildcard: ValueOrWildcard::Wildcard,
            token_stream,
        }
    }
}

impl<T> Display for SortableItem<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = self.token_stream.to_string();
        output = output.replace(" :: ", "::");

        if output.contains(' ') {
            write!(f, "`{output}`")
        } else {
            write!(f, "{output}")
        }
    }
}

impl<T: PartialEq> PartialEq for SortableItem<T> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.value_or_wildcard, &other.value_or_wildcard) {
            (ValueOrWildcard::Wildcard, ValueOrWildcard::Wildcard) => true,
            (ValueOrWildcard::Wildcard, _) | (_, ValueOrWildcard::Wildcard) => false,
            (ValueOrWildcard::Value(self_v), ValueOrWildcard::Value(other_v)) => self_v == other_v,
        }
    }
}

impl<T: Eq> Eq for SortableItem<T> {}

impl<T: PartialOrd> PartialOrd for SortableItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (&self.value_or_wildcard, &other.value_or_wildcard) {
            (ValueOrWildcard::Wildcard, ValueOrWildcard::Wildcard) => Some(Ordering::Equal),
            (ValueOrWildcard::Wildcard, _) => Some(Ordering::Greater),
            (_, ValueOrWildcard::Wildcard) => Some(Ordering::Less),
            (ValueOrWildcard::Value(self_v), ValueOrWildcard::Value(other_v)) => {
                self_v.partial_cmp(other_v)
            }
        }
    }
}

impl<T: Ord> Ord for SortableItem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.value_or_wildcard, &other.value_or_wildcard) {
            (ValueOrWildcard::Wildcard, ValueOrWildcard::Wildcard) => Ordering::Equal,
            (ValueOrWildcard::Wildcard, _) => Ordering::Greater,
            (_, ValueOrWildcard::Wildcard) => Ordering::Less,
            (ValueOrWildcard::Value(self_v), ValueOrWildcard::Value(other_v)) => {
                self_v.cmp(other_v)
            }
        }
    }
}

impl From<Ident> for SortableItem<StringSegments> {
    fn from(ident: Ident) -> Self {
        Self {
            value_or_wildcard: ValueOrWildcard::Value(vec![ident.to_string()]),
            token_stream: ident.into_token_stream(),
        }
    }
}

pub enum SortableItems {
    SignedInts(Vec<SortableItem<i128>>),
    UnsignedInts(Vec<SortableItem<u128>>),
    Floats(Vec<SortableItem<f64>>),
    StringSegments(Vec<SortableItem<StringSegments>>),
    ByteStrings(Vec<SortableItem<ByteString>>),
    CStrings(Vec<SortableItem<CString>>),
    Wildcards(Vec<SortableItem<()>>),
}

impl SortableItems {
    pub fn new_speculative() -> Self {
        Self::new_speculative_with_capacity(0)
    }

    pub fn new_speculative_with_capacity(capacity: usize) -> Self {
        // assume that we will have sortable items with string segments
        SortableItems::StringSegments(Vec::with_capacity(capacity))
    }

    pub fn try_append(
        &mut self,
        value: SortableItemValueEnum,
        token_stream: TokenStream,
    ) -> Result<(), Error> {
        match self {
            SortableItems::SignedInts(items) => {
                if let Some(converted_items) =
                    Self::try_append_to_signed_ints(items, &value, token_stream)?
                {
                    *self = converted_items;
                }
            }
            SortableItems::UnsignedInts(items) => {
                Self::try_append_to_unsigned_ints(items, &value, token_stream)?;
            }
            SortableItems::Floats(items) => {
                Self::try_append_to_floats(items, &value, token_stream)?;
            }
            SortableItems::StringSegments(items) => {
                if let Some(converted_items) =
                    Self::try_append_to_string_segments(items, value, token_stream)?
                {
                    *self = converted_items;
                }
            }
            SortableItems::ByteStrings(items) => {
                Self::try_append_to_byte_strings(items, value, token_stream)?;
            }
            SortableItems::CStrings(items) => {
                Self::try_append_to_c_strings(items, value, token_stream)?;
            }
            SortableItems::Wildcards(items) => {
                if let Some(converted_items) = Self::append_to_wildcards(items, value, token_stream)
                {
                    *self = converted_items;
                }
            }
        }

        Ok(())
    }

    pub fn look_for_unsorted_items(&self) -> Result<(), Error> {
        match self {
            SortableItems::SignedInts(items) => look_for_unsorted_items(items),
            SortableItems::UnsignedInts(items) => look_for_unsorted_items(items),
            SortableItems::Floats(items) => look_for_unsorted_items(items),
            SortableItems::StringSegments(items) => look_for_unsorted_items(items),
            SortableItems::ByteStrings(items) => look_for_unsorted_items(items),
            SortableItems::CStrings(items) => look_for_unsorted_items(items),
            SortableItems::Wildcards(_) => Ok(()),
        }
    }

    fn try_append_to_signed_ints(
        items: &mut Vec<SortableItem<i128>>,
        value: &SortableItemValueEnum,
        token_stream: TokenStream,
    ) -> Result<Option<SortableItems>, Error> {
        match *value {
            SortableItemValueEnum::SignedInt(v) => {
                items.push(SortableItem::new_with_value(v, token_stream));
            }
            SortableItemValueEnum::UnsignedInt(v) => {
                /*
                    The default when parsing an int is to assume that it is signed. Therefore,
                    if we encounter an unsigned int, we need to convert the signed ints that
                    we already parsed to unsigned ones and then append the current value.
                */

                let items = std::mem::take(items);
                let original_len = items.len();

                let mut converted_items: Vec<_> = items
                    .into_iter()
                    .map_while(|original_item| match original_item.value_or_wildcard {
                        ValueOrWildcard::Wildcard => {
                            Some(SortableItem::new_with_wildcard(original_item.token_stream))
                        }
                        ValueOrWildcard::Value(v) => u128::try_from(v)
                            .ok()
                            .map(|i| SortableItem::new_with_value(i, original_item.token_stream)),
                    })
                    .collect();

                if original_len != converted_items.len() {
                    return Err(generate_range_error(&token_stream));
                }

                converted_items.push(SortableItem::new_with_value(v, token_stream));
                return Ok(Some(SortableItems::UnsignedInts(converted_items)));
            }
            SortableItemValueEnum::Float(_)
            | SortableItemValueEnum::StringSegments(_)
            | SortableItemValueEnum::ByteString(_)
            | SortableItemValueEnum::CString(_) => {
                return Err(generate_comparison_error(&token_stream));
            }
            SortableItemValueEnum::Wildcard => {
                items.push(SortableItem::new_with_wildcard(token_stream));
            }
        }

        Ok(None)
    }

    fn try_append_to_unsigned_ints(
        items: &mut Vec<SortableItem<u128>>,
        value: &SortableItemValueEnum,
        token_stream: TokenStream,
    ) -> Result<(), Error> {
        match *value {
            SortableItemValueEnum::SignedInt(v) => {
                /*
                    The default when parsing ints is to assume that they are signed. Therefore,
                    if we are here, it means we previously encountered an unsigned int and we
                    need to convert the current value to be unsigned too.
                */

                let v = u128::try_from(v).map_err(|_| generate_range_error(&token_stream))?;
                items.push(SortableItem::new_with_value(v, token_stream));
            }
            SortableItemValueEnum::UnsignedInt(v) => {
                items.push(SortableItem::new_with_value(v, token_stream));
            }
            SortableItemValueEnum::Float(_)
            | SortableItemValueEnum::StringSegments(_)
            | SortableItemValueEnum::ByteString(_)
            | SortableItemValueEnum::CString(_) => {
                return Err(generate_comparison_error(&token_stream));
            }
            SortableItemValueEnum::Wildcard => {
                items.push(SortableItem::new_with_wildcard(token_stream));
            }
        }

        Ok(())
    }

    fn try_append_to_floats(
        items: &mut Vec<SortableItem<f64>>,
        value: &SortableItemValueEnum,
        token_stream: TokenStream,
    ) -> Result<(), Error> {
        match *value {
            SortableItemValueEnum::SignedInt(_)
            | SortableItemValueEnum::UnsignedInt(_)
            | SortableItemValueEnum::StringSegments(_)
            | SortableItemValueEnum::ByteString(_)
            | SortableItemValueEnum::CString(_) => {
                return Err(generate_comparison_error(&token_stream));
            }
            SortableItemValueEnum::Float(v) => {
                items.push(SortableItem::new_with_value(v, token_stream));
            }
            SortableItemValueEnum::Wildcard => {
                items.push(SortableItem::new_with_wildcard(token_stream));
            }
        }

        Ok(())
    }

    fn try_append_to_string_segments(
        items: &mut Vec<SortableItem<StringSegments>>,
        value: SortableItemValueEnum,
        token_stream: TokenStream,
    ) -> Result<Option<SortableItems>, Error> {
        match value {
            SortableItemValueEnum::SignedInt(v) => {
                if !items.is_empty() {
                    return Err(generate_comparison_error(&token_stream));
                }

                return Ok(Some(SortableItems::SignedInts(vec![
                    SortableItem::new_with_value(v, token_stream),
                ])));
            }
            SortableItemValueEnum::UnsignedInt(v) => {
                if !items.is_empty() {
                    return Err(generate_comparison_error(&token_stream));
                }

                return Ok(Some(SortableItems::UnsignedInts(vec![
                    SortableItem::new_with_value(v, token_stream),
                ])));
            }
            SortableItemValueEnum::Float(v) => {
                if !items.is_empty() {
                    return Err(generate_comparison_error(&token_stream));
                }

                return Ok(Some(SortableItems::Floats(vec![
                    SortableItem::new_with_value(v, token_stream),
                ])));
            }
            SortableItemValueEnum::StringSegments(v) => {
                items.push(SortableItem::new_with_value(v, token_stream));
            }
            SortableItemValueEnum::ByteString(v) => {
                if !items.is_empty() {
                    return Err(generate_comparison_error(&token_stream));
                }

                return Ok(Some(SortableItems::ByteStrings(vec![
                    SortableItem::new_with_value(v, token_stream),
                ])));
            }
            SortableItemValueEnum::CString(v) => {
                if !items.is_empty() {
                    return Err(generate_comparison_error(&token_stream));
                }

                return Ok(Some(SortableItems::CStrings(vec![
                    SortableItem::new_with_value(v, token_stream),
                ])));
            }
            SortableItemValueEnum::Wildcard => {
                if items.is_empty() {
                    return Ok(Some(SortableItems::Wildcards(vec![
                        SortableItem::new_with_wildcard(token_stream),
                    ])));
                }

                items.push(SortableItem::new_with_wildcard(token_stream));
            }
        }

        Ok(None)
    }

    fn try_append_to_byte_strings(
        items: &mut Vec<SortableItem<ByteString>>,
        value: SortableItemValueEnum,
        token_stream: TokenStream,
    ) -> Result<(), Error> {
        match value {
            SortableItemValueEnum::SignedInt(_)
            | SortableItemValueEnum::UnsignedInt(_)
            | SortableItemValueEnum::Float(_)
            | SortableItemValueEnum::StringSegments(_)
            | SortableItemValueEnum::CString(_) => {
                return Err(generate_comparison_error(&token_stream));
            }
            SortableItemValueEnum::ByteString(v) => {
                items.push(SortableItem::new_with_value(v, token_stream));
            }
            SortableItemValueEnum::Wildcard => {
                items.push(SortableItem::new_with_wildcard(token_stream));
            }
        }

        Ok(())
    }

    fn try_append_to_c_strings(
        items: &mut Vec<SortableItem<CString>>,
        value: SortableItemValueEnum,
        token_stream: TokenStream,
    ) -> Result<(), Error> {
        match value {
            SortableItemValueEnum::SignedInt(_)
            | SortableItemValueEnum::UnsignedInt(_)
            | SortableItemValueEnum::Float(_)
            | SortableItemValueEnum::StringSegments(_)
            | SortableItemValueEnum::ByteString(_) => {
                return Err(generate_comparison_error(&token_stream));
            }
            SortableItemValueEnum::CString(v) => {
                items.push(SortableItem::new_with_value(v, token_stream));
            }
            SortableItemValueEnum::Wildcard => {
                items.push(SortableItem::new_with_wildcard(token_stream));
            }
        }

        Ok(())
    }

    fn append_to_wildcards(
        items: &mut Vec<SortableItem<()>>,
        value: SortableItemValueEnum,
        token_stream: TokenStream,
    ) -> Option<SortableItems> {
        match value {
            SortableItemValueEnum::SignedInt(v) => {
                let items = std::mem::take(items)
                    .into_iter()
                    .map(|item| SortableItem::new_with_wildcard(item.token_stream))
                    .chain(std::iter::once(SortableItem::new_with_value(
                        v,
                        token_stream,
                    )))
                    .collect();

                return Some(SortableItems::SignedInts(items));
            }
            SortableItemValueEnum::UnsignedInt(v) => {
                let items = std::mem::take(items)
                    .into_iter()
                    .map(|item| SortableItem::new_with_wildcard(item.token_stream))
                    .chain(std::iter::once(SortableItem::new_with_value(
                        v,
                        token_stream,
                    )))
                    .collect();

                return Some(SortableItems::UnsignedInts(items));
            }
            SortableItemValueEnum::Float(v) => {
                let items = std::mem::take(items)
                    .into_iter()
                    .map(|item| SortableItem::new_with_wildcard(item.token_stream))
                    .chain(std::iter::once(SortableItem::new_with_value(
                        v,
                        token_stream,
                    )))
                    .collect();

                return Some(SortableItems::Floats(items));
            }
            SortableItemValueEnum::StringSegments(v) => {
                let items = std::mem::take(items)
                    .into_iter()
                    .map(|item| SortableItem::new_with_wildcard(item.token_stream))
                    .chain(std::iter::once(SortableItem::new_with_value(
                        v,
                        token_stream,
                    )))
                    .collect();

                return Some(SortableItems::StringSegments(items));
            }
            SortableItemValueEnum::ByteString(v) => {
                let items = std::mem::take(items)
                    .into_iter()
                    .map(|item| SortableItem::new_with_wildcard(item.token_stream))
                    .chain(std::iter::once(SortableItem::new_with_value(
                        v,
                        token_stream,
                    )))
                    .collect();

                return Some(SortableItems::ByteStrings(items));
            }
            SortableItemValueEnum::CString(v) => {
                let items = std::mem::take(items)
                    .into_iter()
                    .map(|item| SortableItem::new_with_wildcard(item.token_stream))
                    .chain(std::iter::once(SortableItem::new_with_value(
                        v,
                        token_stream,
                    )))
                    .collect();

                return Some(SortableItems::CStrings(items));
            }
            SortableItemValueEnum::Wildcard => {
                items.push(SortableItem::new_with_wildcard(token_stream));
            }
        }

        None
    }
}

#[derive(Clone)]
pub enum SortableItemValueEnum {
    SignedInt(i128),
    UnsignedInt(u128),
    Float(f64),
    StringSegments(StringSegments),
    ByteString(ByteString),
    CString(CString),
    Wildcard,
}

impl SortableItemValueEnum {
    pub fn new_speculative() -> Self {
        // assume the value will have string segments
        SortableItemValueEnum::StringSegments(Vec::new())
    }
}

impl From<&Ident> for SortableItemValueEnum {
    fn from(ident: &Ident) -> Self {
        let segments = vec![ident.to_string()];
        SortableItemValueEnum::StringSegments(segments)
    }
}

impl From<&LitStr> for SortableItemValueEnum {
    fn from(lit_str: &LitStr) -> Self {
        let segments = vec![lit_str.value()];
        SortableItemValueEnum::StringSegments(segments)
    }
}

impl From<&LitByteStr> for SortableItemValueEnum {
    fn from(lit_byte_str: &LitByteStr) -> Self {
        SortableItemValueEnum::ByteString(lit_byte_str.value())
    }
}

impl From<&LitCStr> for SortableItemValueEnum {
    fn from(lit_c_str: &LitCStr) -> Self {
        SortableItemValueEnum::CString(lit_c_str.value())
    }
}

impl From<&LitByte> for SortableItemValueEnum {
    fn from(lit_byte: &LitByte) -> Self {
        SortableItemValueEnum::SignedInt(i128::from(lit_byte.value()))
    }
}

impl From<&LitChar> for SortableItemValueEnum {
    fn from(lit_char: &LitChar) -> Self {
        let segments = vec![lit_char.value().to_string()];
        SortableItemValueEnum::StringSegments(segments)
    }
}

impl From<&LitBool> for SortableItemValueEnum {
    fn from(lit_bool: &LitBool) -> Self {
        SortableItemValueEnum::SignedInt(i128::from(lit_bool.value))
    }
}

impl TryFrom<&LitInt> for SortableItemValueEnum {
    type Error = syn::Error;

    fn try_from(lit_int: &LitInt) -> Result<Self, Self::Error> {
        if let Ok(signed_int) = lit_int.base10_parse::<i128>() {
            Ok(SortableItemValueEnum::SignedInt(signed_int))
        } else {
            Ok(SortableItemValueEnum::UnsignedInt(
                lit_int.base10_parse::<u128>()?,
            ))
        }
    }
}

impl TryFrom<&LitFloat> for SortableItemValueEnum {
    type Error = syn::Error;

    fn try_from(lit_float: &LitFloat) -> Result<Self, Self::Error> {
        Ok(SortableItemValueEnum::Float(lit_float.base10_parse()?))
    }
}

impl From<&Path> for SortableItemValueEnum {
    fn from(path: &Path) -> Self {
        let num_segments = path.segments.len() + usize::from(path.leading_colon.is_some());
        let mut segments = Vec::with_capacity(num_segments);

        if path.leading_colon.is_some() {
            segments.push(String::new());
        }

        for segment in &path.segments {
            segments.push(segment.ident.to_string());
        }

        SortableItemValueEnum::StringSegments(segments)
    }
}

fn look_for_unsorted_items<T: PartialOrd>(sortable_items: &[SortableItem<T>]) -> Result<(), Error> {
    for (index, item) in sortable_items.iter().enumerate().skip(1) {
        if sortable_items[index - 1] > *item {
            let partition_point =
                sortable_items[..index].partition_point(|checked_item| checked_item <= item);

            return Err(Error::new_spanned(
                &item.token_stream,
                format!(
                    "{item} should sort before {}",
                    sortable_items[partition_point]
                ),
            ));
        }
    }

    Ok(())
}

fn generate_range_error(token_stream: &TokenStream) -> Error {
    Error::new_spanned(
        token_stream,
        "value does not fit into the range inferred from the previous patterns",
    )
}

fn generate_comparison_error(token_stream: &TokenStream) -> Error {
    Error::new_spanned(
        token_stream,
        "comparison between this pattern and the previously encountered patterns is unsupported by #[sorted]"
    )
}
