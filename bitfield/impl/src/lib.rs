//! Provides internal implementation details for the `bitfield` crate.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    ext::IdentExt, parse2, parse_macro_input, punctuated::Punctuated, spanned::Spanned, Attribute,
    Error, Field, Fields, Ident, ItemEnum, ItemStruct, LitInt, Path, Token, Type, Variant,
};

#[proc_macro_attribute]
pub fn bitfield(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = args;
    let item = parse_macro_input!(input as ItemStruct);

    match parse_struct_item(&item) {
        Ok(stream) => stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(BitfieldSpecifier, attributes(bits, set_bits))]
pub fn bitfield_specifier(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as ItemEnum);

    match implement_specifier(&item) {
        Ok(stream) => stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[doc(hidden)]
#[proc_macro]
pub fn generate_bit_specifiers(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if !input.is_empty() {
        return Error::new_spanned(TokenStream::from(input), "arguments are not allowed")
            .to_compile_error()
            .into();
    }

    let items = (1u8..=MAX_BITS).map(|size| {
        let ident = get_bit_specifier_ident(size);
        let field_type = get_primitive_type(size);
        let doc_msg = format!("Specifies that a field should occupy {size} {}.", if size == 1 { "bit" } else { "bits" });

        quote! {
            #[doc = #doc_msg]
            pub enum #ident {}

            #[doc(hidden)]
            impl Specifier for #ident {
                const BITS: ::core::num::NonZero<u8> = ::core::num::NonZero::new(#size).expect("size should be greater than 0");

                type GetType = #field_type;
                type SetType = #field_type;

                fn get(data_storage: &[u8], bit_index: usize) -> Self::GetType {
                    let mut bit_index_in_storage = bit_index;
                    let mut bit_index_in_value = 0;
                    let mut value: #field_type = 0;

                    loop {
                        let byte_index = bit_index_in_storage / 8;

                        let bit_offset = u8::try_from(bit_index_in_storage % 8)
                            .expect("bit_offset should be in range for u8");

                        let bits_in_this_byte = u8::min(8 - bit_offset, Self::BITS.get() - bit_index_in_value);
                        let data_byte_mask = 1u8.unbounded_shl(bits_in_this_byte.into()).wrapping_sub(1);

                        let data_byte = &data_storage[byte_index];
                        let bits = (data_byte >> bit_offset) & data_byte_mask;

                        value |= #field_type::from(bits) << bit_index_in_value;
                        bit_index_in_value += bits_in_this_byte;

                        if bit_index_in_value == Self::BITS.get() {
                            break;
                        }

                        bit_index_in_storage += usize::from(bits_in_this_byte);
                    }

                    value
                }

                fn set(data_storage: &mut [u8], bit_index: usize, mut value: Self::SetType) {
                    let mut bit_index_in_storage = bit_index;
                    let mut bit_index_in_value = 0;

                    loop {
                        let byte_index = bit_index_in_storage / 8;

                        let bit_offset = u8::try_from(bit_index_in_storage % 8)
                            .expect("bit_offset should be in range for u8");

                        let bits_in_this_byte = u8::min(8 - bit_offset, Self::BITS.get() - bit_index_in_value);

                        let mask = 1u8.unbounded_shl(bits_in_this_byte.into()).wrapping_sub(1);
                        let value_mask = #field_type::from(mask);

                        let bits = u8::try_from(value & value_mask).expect("bits should be in range for u8");

                        let data_byte = &mut data_storage[byte_index];
                        let data_byte_mask = !(mask << bit_offset);

                        *data_byte &= data_byte_mask;
                        *data_byte |= bits << bit_offset;

                        bit_index_in_value += bits_in_this_byte;

                        if bit_index_in_value == Self::BITS.get() {
                            break;
                        }

                        bit_index_in_storage += usize::from(bits_in_this_byte);
                        value >>= bits_in_this_byte;
                    }
                }
            }
        }
    });

    quote! {
        #(#items)*
    }
    .into()
}

fn parse_struct_item(item: &ItemStruct) -> Result<TokenStream, Error> {
    let vis = &item.vis;
    let ident = &item.ident;
    let generics = &item.generics;
    let semi_token = item.semi_token;

    let output_token_streams = get_output_token_streams(item)?;

    let struct_attrs = output_token_streams.struct_attrs;
    let struct_def = output_token_streams.struct_def;
    let struct_impl = output_token_streams.struct_impl;
    let documented_size_check = output_token_streams.documented_size_check;
    let multiple_of_8_size_check = output_token_streams.multiple_of_8_size_check;
    let field_size_checks = output_token_streams.field_size_checks;

    Ok(quote! {
        #(#struct_attrs)*
        #vis struct #ident #generics
        #struct_def #semi_token

        #struct_impl

        #documented_size_check
        #multiple_of_8_size_check

        #(#field_size_checks)*
    })
}

struct OutputTokenStreams {
    struct_attrs: Vec<TokenStream>,
    struct_def: TokenStream,
    struct_impl: TokenStream,
    documented_size_check: Option<TokenStream>,
    multiple_of_8_size_check: TokenStream,
    field_size_checks: Vec<TokenStream>,
}

fn get_output_token_streams(item: &ItemStruct) -> Result<OutputTokenStreams, Error> {
    let output_token_streams = match &item.fields {
        Fields::Named(fields) => {
            let storage_access = quote! {
                &self.data
            };

            let storage_access_mut = quote! {
                &mut self.data
            };

            let intermediate_token_streams = get_intermediate_token_streams(
                &fields.named,
                &item.ident,
                &storage_access,
                &storage_access_mut,
            )?;

            let total_bits = intermediate_token_streams.total_bits;
            let accessors = intermediate_token_streams.accessors;
            let field_size_checks = intermediate_token_streams.field_size_checks;

            let struct_def = quote! {
                { data: [::core::primitive::u8; (#total_bits)/8] }
            };

            let ident = &item.ident;
            let new_fn_doc_msg = format!(
                "Creates a new instance of [`{ident}`](struct@{ident}) with all bits set to 0."
            );

            let struct_impl = quote! {
                impl #ident {
                    #[doc = #new_fn_doc_msg]
                    pub fn new() -> Self {
                        Self { data: [0; (#total_bits)/8] }
                    }

                    #accessors
                }
            };

            let parsed_attributes = parse_struct_attributes(&item.attrs, &item.ident, &total_bits)?;
            let struct_attrs = parsed_attributes.attributes;
            let documented_size_check = parsed_attributes.documented_size_check;

            let multiple_of_8_size_check = get_multiple_of_8_size_check(item, &total_bits);

            OutputTokenStreams {
                struct_attrs,
                struct_def,
                struct_impl,
                documented_size_check,
                multiple_of_8_size_check,
                field_size_checks,
            }
        }
        Fields::Unnamed(fields) => {
            let storage_access = quote! {
                &self.0
            };

            let storage_access_mut = quote! {
                &mut self.0
            };

            let intermediate_token_streams = get_intermediate_token_streams(
                &fields.unnamed,
                &item.ident,
                &storage_access,
                &storage_access_mut,
            )?;

            let total_bits = intermediate_token_streams.total_bits;
            let accessors = intermediate_token_streams.accessors;
            let field_size_checks = intermediate_token_streams.field_size_checks;

            let struct_def = quote! {
                ([::core::primitive::u8; (#total_bits)/8])
            };

            let ident = &item.ident;

            let struct_impl = quote! {
                impl #ident {
                    pub fn new() -> Self {
                        Self([0; (#total_bits)/8])
                    }

                    #accessors
                }
            };

            let parsed_attributes = parse_struct_attributes(&item.attrs, &item.ident, &total_bits)?;
            let struct_attrs = parsed_attributes.attributes;
            let documented_size_check = parsed_attributes.documented_size_check;

            let multiple_of_8_size_check = get_multiple_of_8_size_check(item, &total_bits);

            OutputTokenStreams {
                struct_attrs,
                struct_def,
                struct_impl,
                documented_size_check,
                multiple_of_8_size_check,
                field_size_checks,
            }
        }
        Fields::Unit => {
            return Err(Error::new_spanned(
                &item.ident,
                "expected struct to have at least one field",
            ));
        }
    };

    Ok(output_token_streams)
}

struct IntermediateTokenStreams {
    total_bits: TokenStream,
    accessors: TokenStream,
    field_size_checks: Vec<TokenStream>,
}

fn get_intermediate_token_streams(
    fields: &Punctuated<Field, Token![,]>,
    struct_ident: &Ident,
    storage_access: &TokenStream,
    storage_access_mut: &TokenStream,
) -> Result<IntermediateTokenStreams, Error> {
    let mut bit_index = quote! { 0 };
    let mut accessors = Vec::with_capacity(fields.len());
    let mut field_size_checks = Vec::new();

    for (field_index, field) in fields.iter().enumerate() {
        let (field_ident, field_span) = match field.ident.as_ref() {
            Some(ident) => (ident.unraw().to_string(), ident.span()),
            None => (field_index.to_string(), field.ty.span()),
        };

        if let Some(check) = get_field_size_check(
            &field.attrs,
            &field.ty,
            &field_ident,
            field_span,
            struct_ident,
        )? {
            field_size_checks.push(check);
        }

        let get_fn_doc_msg = format!("Gets the value of field `{field_ident}`.");
        let get_fn_name = format_ident!("get_{field_ident}");

        let set_fn_doc_msg = format!("Sets the value of field `{field_ident}`.");
        let set_fn_name = format_ident!("set_{field_ident}");

        let vis = &field.vis;
        let ty_as_specifier = get_specifier(&field.ty);

        let field_accessors = quote! {
            #[doc = #get_fn_doc_msg]
            #vis fn #get_fn_name(&self) -> #ty_as_specifier::GetType {
                #ty_as_specifier::get(#storage_access, #bit_index)
            }

            #[doc = #set_fn_doc_msg]
            #vis fn #set_fn_name(&mut self, value: #ty_as_specifier::SetType) {
                #ty_as_specifier::set(#storage_access_mut, #bit_index, value);
            }
        };

        accessors.push(field_accessors);
        bit_index.extend(quote! { + #ty_as_specifier::BITS.get() as ::core::primitive::usize });
    }

    let accessors = quote! {
        #(#accessors)*
    };

    Ok(IntermediateTokenStreams {
        total_bits: bit_index,
        accessors,
        field_size_checks,
    })
}

struct ParsedStructAttributes {
    documented_size_check: Option<TokenStream>,
    attributes: Vec<TokenStream>,
}

fn parse_struct_attributes(
    attrs: &[Attribute],
    struct_ident: &Ident,
    total_bits: &TokenStream,
) -> Result<ParsedStructAttributes, Error> {
    let mut documented_size_check = None;
    let mut attributes = Vec::new();

    for attr in attrs {
        if is_bits_attribute(attr.path()) {
            if documented_size_check.is_some() {
                return Err(Error::new_spanned(
                    attr,
                    "multiple `bits` attributes on the same struct",
                ));
            }

            let bits = parse_attribute_as_usize(attr)?;
            let ident = format_ident!(
                "size_of_struct_{struct_ident}_is_equal_to_the_value_of_the_bits_attribute"
            );

            documented_size_check = Some(quote_spanned! {struct_ident.span()=>
                #[allow(dead_code, non_upper_case_globals)]
                #[doc(hidden)]
                const #ident: () = {
                    ::core::assert!((#total_bits) == #bits, "expected struct size to be equal to the value of the `bits` attribute");
                };
            });
        } else {
            attributes.push(attr.to_token_stream());
        }
    }

    Ok(ParsedStructAttributes {
        documented_size_check,
        attributes,
    })
}

fn get_field_size_check(
    field_attrs: &[Attribute],
    field_ty: &Type,
    field_ident: &str,
    field_span: Span,
    struct_ident: &Ident,
) -> Result<Option<TokenStream>, Error> {
    let mut check = None;

    for attr in field_attrs {
        if is_bits_attribute(attr.path()) {
            if check.is_some() {
                return Err(Error::new_spanned(
                    attr,
                    "multiple `bits` attributes on the same field",
                ));
            }

            let ident = format_ident!(
                "size_of_field_{field_ident}_of_struct_{struct_ident}_is_equal_to_the_value_of_the_bits_attribute",
            );

            let bits = parse_attribute_as_u8(attr)?;
            let field_as_specifier = get_specifier(field_ty);

            /*
                While the original test wanted us to access an array out of bounds to trigger a compile error
                that also shows the correct number of bits, I find that error confusing as it talks about the
                array specifically. While the implementation I have chosen does not show the correct number of bits,
                it is, in my opinion, showing clearly where the problem lies.
            */

            check = Some(quote_spanned! {field_span=>
                #[allow(dead_code, non_upper_case_globals)]
                #[doc(hidden)]
                const #ident: () = {
                    ::core::assert!(
                        #field_as_specifier::BITS.get() == #bits,
                        "expected field size to be equal to the value of the `bits` attribute",
                    );
                };
            });
        }
    }

    Ok(check)
}

fn get_multiple_of_8_size_check(item: &ItemStruct, total_bits: &TokenStream) -> TokenStream {
    /*
        While the original test wanted us to implement the solution described
        in https://github.com/dtolnay/case-studies/blob/master/bitfield-assertion/README.md,
        I find the error that it produces quite unreadable. Therefore, even if the solution
        I have chosen here might not produce a compile error during 'cargo check', what it
        outputs during 'cargo build' as an error is much, much more user-friendly.
    */

    let ident = format_ident!("size_of_{}_is_multiple_of_8_bits", &item.ident);
    let span = item.ident.span();

    quote_spanned! {span=>
        #[allow(dead_code, non_upper_case_globals)]
        #[doc(hidden)]
        const #ident: () = {
            ::core::assert!((#total_bits).is_multiple_of(8), "expected struct size to be a multiple of 8 bits");
        };
    }
}

fn implement_specifier(item: &ItemEnum) -> Result<TokenStream, Error> {
    let ident = &item.ident;
    let parsed_enum = parse_enum(item)?;

    let bits = parsed_enum.bits;
    let bit_specifier_as_specifier = &parsed_enum.bit_specifier_as_specifier;
    let get_type = &parsed_enum.get_type;
    let get_match_body = &parsed_enum.get_match_body;
    let range_checks = &parsed_enum.range_checks;
    let variants = &parsed_enum.variants;

    let set_match_arms = variants.iter().map(|variant| {
        let path = &variant.path;
        let value_as_integer = &variant.value_as_integer;

        quote! {
            #path => #value_as_integer,
        }
    });

    Ok(quote! {
        #range_checks

        #[doc(hidden)]
        impl bitfield::Specifier for #ident {
            const BITS: ::core::num::NonZero<u8> = ::core::num::NonZero::new(#bits).expect("bits should be greater than 0");

            type GetType = #get_type;
            type SetType = #ident;

            fn get(data_storage: &[::core::primitive::u8], bit_index: ::core::primitive::usize) -> Self::GetType {
                let value = #bit_specifier_as_specifier::get(data_storage, bit_index);

                match value {
                    #get_match_body
                }
            }

            fn set(data_storage: &mut [::core::primitive::u8], bit_index: ::core::primitive::usize, value: Self::SetType) {
                let value = match value {
                    #(#set_match_arms)*
                };

                #bit_specifier_as_specifier::set(data_storage, bit_index, value);
            }
        }
    })
}

fn parse_attribute_as_usize(attr: &Attribute) -> Result<usize, Error> {
    let name_value = attr.meta.require_name_value()?;
    let value = parse_as_u128(&name_value.value)?;

    usize::try_from(value).map_err(|_| {
        Error::new_spanned(
            &name_value.value,
            format!("expected no more than {} bits", usize::MAX),
        )
    })
}

fn parse_attribute_as_u8(attr: &Attribute) -> Result<u8, Error> {
    let name_value = attr.meta.require_name_value()?;
    let value = parse_as_u128(&name_value.value)?;

    u8::try_from(value)
        .ok()
        .and_then(|bits| if bits > MAX_BITS { None } else { Some(bits) })
        .ok_or_else(|| {
            Error::new_spanned(
                &name_value.value,
                format!("expected no more than {MAX_BITS} bits"),
            )
        })
}

struct ParsedEnum {
    bits: u8,
    bit_specifier_as_specifier: TokenStream,
    get_type: TokenStream,
    get_match_body: TokenStream,
    range_checks: TokenStream,
    variants: Vec<ParsedEnumVariant>,
}

fn parse_enum(item: &ItemEnum) -> Result<ParsedEnum, Error> {
    let ident = &item.ident;
    let parsed_attributes = parse_enum_attributes(&item.attrs)?;

    let parsed_enum = if let Some(bits) = parsed_attributes.set_bits {
        ensure_enum_size_matches_documented_bits(bits, parsed_attributes.documented_bits, ident)?;

        let bit_specifier_as_specifier = get_specifier(get_bit_specifier_ident(bits));

        let get_type = quote! {
            ::core::result::Result<#ident, ::bitfield::Unrecognized<#bit_specifier_as_specifier::GetType>>
        };

        let primitive_type = get_primitive_type(bits);
        let max_representable_value = 2u128.pow(bits.into());
        let variants = parse_enum_variants(item, &primitive_type, max_representable_value)?;

        let get_match_arms = variants.iter().map(|variant| {
            let path = &variant.path;
            let value_as_integer = &variant.value_as_integer;

            quote! {
                v if v == #value_as_integer => ::core::result::Result::Ok(#path),
            }
        });

        let get_match_body = quote! {
            #(#get_match_arms)*
            v => ::core::result::Result::Err(::bitfield::Unrecognized::new(v)),
        };

        let range_checks = get_range_checks(&variants);

        ParsedEnum {
            bits,
            bit_specifier_as_specifier,
            get_type,
            get_match_body,
            range_checks,
            variants,
        }
    } else {
        let num_variants = item.variants.len();

        if !num_variants.is_power_of_two() {
            return Err(Error::new_spanned(
                ident,
                "expected a number of variants which is a power of 2",
            ));
        }

        let bits = u8::try_from(num_variants.ilog2())
            .ok()
            .and_then(|bits| if bits > MAX_BITS { None } else { Some(bits) })
            .ok_or_else(|| {
                Error::new_spanned(
                    ident,
                    format!("expected no more than {} variants", u128::MAX),
                )
            })?;

        ensure_enum_size_matches_documented_bits(bits, parsed_attributes.documented_bits, ident)?;

        let bit_specifier_as_specifier = get_specifier(get_bit_specifier_ident(bits));
        let get_type = quote! { #ident };
        let primitive_type = get_primitive_type(bits);

        let max_representable_value = num_variants
            .try_into()
            .expect("num_variants should be in range for u128");

        let variants = parse_enum_variants(item, &primitive_type, max_representable_value)?;

        let get_match_arms = variants.iter().map(|variant| {
            let path = &variant.path;
            let value_as_integer = &variant.value_as_integer;

            quote! {
                v if v == #value_as_integer => #path,
            }
        });

        let get_match_body = quote! {
            #(#get_match_arms)*
            _ => ::core::unreachable!("enum variants should be exhaustive"),
        };

        let range_checks = get_range_checks(&variants);

        ParsedEnum {
            bits,
            bit_specifier_as_specifier,
            get_type,
            get_match_body,
            range_checks,
            variants,
        }
    };

    Ok(parsed_enum)
}

struct ParsedEnumAttributes {
    documented_bits: Option<u8>,
    set_bits: Option<u8>,
}

fn parse_enum_attributes(attrs: &[Attribute]) -> Result<ParsedEnumAttributes, Error> {
    let mut documented_bits = None;
    let mut set_bits = None;

    for attr in attrs {
        if is_bits_attribute(attr.path()) {
            if documented_bits.is_some() {
                return Err(Error::new_spanned(
                    attr,
                    "multiple `bits` attributes on the same enum",
                ));
            }

            documented_bits = Some(parse_attribute_as_u8(attr)?);
        } else if is_set_bits_attribute(attr.path()) {
            if set_bits.is_some() {
                return Err(Error::new_spanned(
                    attr,
                    "multiple `set_bits` attributes on the same enum",
                ));
            }

            set_bits = Some(parse_attribute_as_u8(attr)?);
        }
    }

    Ok(ParsedEnumAttributes {
        documented_bits,
        set_bits,
    })
}

struct ParsedEnumVariant {
    path: TokenStream,
    value_as_integer: TokenStream,
    range_check: TokenStream,
}

fn parse_enum_variants(
    item: &ItemEnum,
    primitive_type: &TokenStream,
    max_representable_value: u128,
) -> Result<Vec<ParsedEnumVariant>, Error> {
    let ident = &item.ident;

    item.variants
        .iter()
        .map(|variant| parse_enum_variant(variant, ident, primitive_type, max_representable_value))
        .collect::<Result<Vec<_>, _>>()
}

fn parse_enum_variant(
    variant: &Variant,
    enum_ident: &Ident,
    enum_primitive_type: &TokenStream,
    max_representable_value: u128,
) -> Result<ParsedEnumVariant, Error> {
    let variant_ident = &variant.ident;

    if !variant.fields.is_empty() {
        return Err(Error::new_spanned(
            variant_ident,
            "expected variant with no fields",
        ));
    }

    let path = quote! { #enum_ident::#variant_ident };
    let value_as_integer = quote! { #path as #enum_primitive_type };
    let check_ident = format_ident!("variant_{variant_ident}_of_enum_{enum_ident}_is_in_bit_range");

    let range_check = quote_spanned! {variant_ident.span()=>
        #[allow(dead_code, non_upper_case_globals)]
        #[doc(hidden)]
        const #check_ident: () = {
            ::core::assert!((#path as u128) < #max_representable_value, "expected variant value to fit inside bit range");
        };
    };

    Ok(ParsedEnumVariant {
        path,
        value_as_integer,
        range_check,
    })
}

fn ensure_enum_size_matches_documented_bits(
    actual_bits: u8,
    documented_bits: Option<u8>,
    enum_ident: &Ident,
) -> Result<(), Error> {
    let Some(documented_bits) = documented_bits else {
        return Ok(());
    };

    if actual_bits == documented_bits {
        Ok(())
    } else {
        Err(Error::new_spanned(
            enum_ident,
            "expected enum size to be equal to the value of the `bits` attribute",
        ))
    }
}

fn is_bits_attribute(path: &Path) -> bool {
    path.is_ident("bits")
}

fn is_set_bits_attribute(path: &Path) -> bool {
    path.is_ident("set_bits")
}

fn get_range_checks(variants: &[ParsedEnumVariant]) -> TokenStream {
    let checks = variants.iter().map(|variant| &variant.range_check);

    quote! {
        #(#checks)*
    }
}

fn get_bit_specifier_ident(size: u8) -> Ident {
    format_ident!("B{size}")
}

fn get_primitive_type(bits: u8) -> TokenStream {
    match bits {
        1..=8 => quote! { ::core::primitive::u8 },
        9..=16 => quote! { ::core::primitive::u16 },
        17..=32 => quote! { ::core::primitive::u32 },
        33..=64 => quote! { ::core::primitive::u64 },
        _ => quote! { ::core::primitive::u128 },
    }
}

fn get_specifier<T: Spanned + ToTokens>(t: T) -> TokenStream {
    let span = t.span();

    quote_spanned! {span=>
        <#t as bitfield::Specifier>
    }
}

fn parse_as_u128<T: ToTokens>(t: T) -> Result<u128, Error> {
    let lit: LitInt = parse2(t.to_token_stream())?;
    lit.base10_parse::<u128>()
}

const MAX_BITS: u8 = 128;
