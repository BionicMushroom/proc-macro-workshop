//! Provides the [`CustomDebug`] derive macro.

use std::collections::{hash_map::Entry, HashMap, HashSet};

use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned, token::Comma,
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Expr, Field, GenericArgument,
    Generics, Ident, ImplGenerics, Index, Lit, LitStr, Meta, MetaList, Path, PathArguments,
    PathSegment, PredicateLifetime, PredicateType, Type, TypeGenerics, WhereClause, WherePredicate,
};

/// A derive macro that implements the [`std::fmt::Debug`](https://doc.rust-lang.org/std/fmt/trait.Debug.html)
/// trait which is more customizable than the similar `derive(Debug)` macro from the standard library.
/// 
/// With it, you can specify:
/// - the formatting used for individual struct fields by providing a format string in the style expected by
///   Rust string formatting macros like [`format!`](https://doc.rust-lang.org/std/macro.format.html)
///   and [`println!`](https://doc.rust-lang.org/std/macro.println.html);
/// - the bounds that the generic arguments of a struct or enum must implement so that the struct or enum itself can
///   be debug-printed.
///
/// # Examples
///
/// ## `derive(CustomDebug)` on a simple struct and enum
///
/// ```
/// use derive_debug::CustomDebug;
///
/// #[derive(CustomDebug)]
/// struct DebugPrintableStruct {
///     field: u32,
/// }
///
/// #[derive(CustomDebug)]
/// enum DebugPrintableEnum {
///     Variant(u32),
/// }
///
/// assert!(
///     format!("{:?}", DebugPrintableStruct { field: 0 }) == "DebugPrintableStruct { field: 0 }"
/// );
///
/// assert!(format!("{:?}", DebugPrintableEnum::Variant(0)) == "Variant(0)");
/// ```
///
/// ## Specifying a format string for a struct or enum field
///
/// ```
/// use derive_debug::CustomDebug;
///
/// #[derive(CustomDebug)]
/// struct DebugPrintableStruct {
///     #[debug = "0b{:08b}"]
///     field: u32,
/// }
///
/// #[derive(CustomDebug)]
/// enum DebugPrintableEnum {
///     Variant(#[debug = "0b{:08b}"] u32),
/// }
///
/// assert!(
///     format!("{:?}", DebugPrintableStruct { field: 16 })
///         == "DebugPrintableStruct { field: 0b00010000 }"
/// );
///
/// assert!(format!("{:?}", DebugPrintableEnum::Variant(16)) == "Variant(0b00010000)");
/// ```
///
/// ## `derive(CustomDebug)` on a generic struct and enum
///
/// ```
/// use derive_debug::CustomDebug;
///
/// #[derive(CustomDebug)]
/// struct DebugPrintableStruct<T> {
///     field: T,
/// }
///
/// #[derive(CustomDebug)]
/// enum DebugPrintableEnum<T> {
///     Variant(T),
/// }
///
/// assert!(
///     format!("{:?}", DebugPrintableStruct { field: 0 }) == "DebugPrintableStruct { field: 0 }"
/// );
///
/// assert!(format!("{:?}", DebugPrintableEnum::Variant(0)) == "Variant(0)");
/// ```
///
/// Note that the macro attempts to infer the correct bounds of the generic arguments so that the
/// struct or the enum itself can be debug-printed. If the bounds inferred by the macro are not
/// correct, you can specify them explicitly as seen in the next example.
///
/// ## Specifying the bounds for a struct or enum generic argument
///
/// ```
/// use derive_debug::CustomDebug;
///
/// trait Trait {
///     type Value;
/// }
///
/// impl Trait for u32 {
///     type Value = u32;
/// }
///
/// #[derive(CustomDebug)]
/// #[debug(bound = "T::Value: std::fmt::Debug")]
/// struct DebugPrintableStruct<T: Trait> {
///     field: Field<T>,
/// }
///
/// #[derive(CustomDebug)]
/// struct AnotherDebugPrintableStruct<T: Trait> {
///     #[debug(bound = "T::Value: std::fmt::Debug")]
///     field: Field<T>,
/// }
///
/// #[derive(CustomDebug)]
/// #[debug(bound = "T::Value: std::fmt::Debug")]
/// enum DebugPrintableEnum<T: Trait> {
///     Variant(Field<T>),
/// }
///
/// #[derive(CustomDebug)]
/// enum AnotherDebugPrintableEnum<T: Trait> {
///     Variant(#[debug(bound = "T::Value: std::fmt::Debug")] Field<T>),
/// }
///
/// #[derive(CustomDebug)]
/// struct Field<T: Trait> {
///     values: Vec<T::Value>,
/// }
///
/// assert!(
///     format!(
///         "{:?}",
///         DebugPrintableStruct {
///             field: Field::<u32> { values: vec![] }
///         }
///     ) == "DebugPrintableStruct { field: Field { values: [] } }"
/// );
///
/// assert!(
///     format!(
///         "{:?}",
///         AnotherDebugPrintableStruct {
///             field: Field::<u32> { values: vec![] }
///         }
///     ) == "AnotherDebugPrintableStruct { field: Field { values: [] } }"
/// );
///
/// assert!(
///     format!(
///         "{:?}",
///         DebugPrintableEnum::Variant(Field::<u32> { values: vec![] })
///     ) == "Variant(Field { values: [] })"
/// );
///
/// assert!(
///     format!(
///         "{:?}",
///         AnotherDebugPrintableEnum::Variant(Field::<u32> { values: vec![] })
///     ) == "Variant(Field { values: [] })"
/// );
/// ```
///
/// Note that the bounds can be specified on the struct itself, on a struct field
/// (if they haven't been specified on the struct itself), on an enum and on an
/// enum field (if they haven't been specified on the enum itself). They can not
/// be specified on an enum variant.
#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match convert_input_to_output(input) {
        Ok(stream) => stream,
        Err(err) => err.to_compile_error().into(),
    }
}

fn convert_input_to_output(input: DeriveInput) -> Result<TokenStream, Error> {
    let input_data = get_input_data(input)?;

    match input_data.kind_dependent {
        KindDependentInputData::Struct(input) => {
            convert_struct_input_to_output(&input, input_data.kind_agnostic)
        }
        KindDependentInputData::Enum(input) => {
            convert_enum_input_to_output(&input, input_data.kind_agnostic)
        }
    }
}

struct InputData {
    kind_agnostic: KindAgnosticInputData,
    kind_dependent: KindDependentInputData,
}

struct KindAgnosticInputData {
    generics: Generics,
    where_predicates: HashSet<WherePredicate>,
    is_where_predicates_adjustment_needed: bool,
    generic_ty_idents: HashSet<Ident>,
    caller_ty: Ident,
}

enum KindDependentInputData {
    Struct(StructInputData),
    Enum(EnumInputData),
}

struct StructInputData {
    data_struct: DataStruct,
}

struct EnumInputData {
    data_enum: DataEnum,
}

fn get_input_data(input: DeriveInput) -> Result<InputData, Error> {
    match input.data {
        Data::Struct(data) => Ok(InputData {
            kind_agnostic: get_kind_agnostic_input_data(
                input.generics,
                input.ident,
                &input.attrs,
                AttrsProvenance::Struct,
            )?,
            kind_dependent: KindDependentInputData::Struct(StructInputData { data_struct: data }),
        }),
        Data::Enum(data) => Ok(InputData {
            kind_agnostic: get_kind_agnostic_input_data(
                input.generics,
                input.ident,
                &input.attrs,
                AttrsProvenance::Enum,
            )?,
            kind_dependent: KindDependentInputData::Enum(EnumInputData { data_enum: data }),
        }),
        Data::Union(data) => Err(Error::new(
            data.union_token.span,
            "expected struct or enum, found `union`",
        )),
    }
}

fn get_kind_agnostic_input_data(
    mut generics: Generics,
    caller_ty: Ident,
    attrs: &[Attribute],
    attrs_provenance: AttrsProvenance,
) -> Result<KindAgnosticInputData, Error> {
    let (mut where_predicates, generic_ty_idents) =
        get_where_predicates_and_generic_ty_idents(&mut generics);

    let parsed_debug_bound_attr =
        parse_debug_attrs(attrs, attrs_provenance, DebugBoundAttributeLegality::Legal)?
            .parsed_debug_bound_attr;

    let is_where_predicates_adjustment_needed =
        if let Some(parsed_debug_bound_attr) = parsed_debug_bound_attr {
            add_where_predicates_to_set(
                parsed_debug_bound_attr.where_predicates,
                &mut where_predicates,
            );

            false
        } else {
            true
        };

    Ok(KindAgnosticInputData {
        generics,
        where_predicates,
        is_where_predicates_adjustment_needed,
        generic_ty_idents,
        caller_ty,
    })
}

fn get_where_predicates_and_generic_ty_idents(
    generics: &mut Generics,
) -> (HashSet<WherePredicate>, HashSet<Ident>) {
    let mut where_predicates = HashSet::new();

    if let Some(where_clause) = generics.where_clause.take() {
        add_where_predicates_to_set(where_clause.predicates, &mut where_predicates);
    }

    let mut generic_ty_idents = HashSet::new();

    for type_param in generics.type_params_mut() {
        let ident = &type_param.ident;
        generic_ty_idents.insert(ident.clone());

        for bound in &type_param.bounds {
            where_predicates.insert(parse_quote!(#ident: #bound));
        }

        type_param.bounds.clear();
    }

    (where_predicates, generic_ty_idents)
}

#[derive(Copy, Clone, PartialEq)]
enum AttrsProvenance {
    Struct,
    StructField,
    Enum,
    EnumField,
    EnumVariant,
}

#[derive(Copy, Clone, PartialEq)]
enum DebugBoundAttributeLegality {
    Legal,
    IllegalAlreadySpecifiedOnStruct,
    IllegalAlreadySpecifiedOnStructFieldType,
    IllegalAlreadySpecifiedOnEnum,
    IllegalNotAllowedOnEnumVariant,
    IllegalAlreadySpecifiedOnEnumFieldType,
}

struct ParsedDebugAttrs<'a> {
    parsed_debug_format_attr: Option<ParsedDebugFormatAttr<'a>>,
    parsed_debug_bound_attr: Option<ParsedDebugBoundAttr>,
}

struct ParsedDebugFormatAttr<'a> {
    format_str: &'a LitStr,
}

struct ParsedDebugBoundAttr {
    where_predicates: Punctuated<WherePredicate, Comma>,
}

fn parse_debug_attrs(
    attrs: &[Attribute],
    provenance: AttrsProvenance,
    bound_attribute_legality: DebugBoundAttributeLegality,
) -> Result<ParsedDebugAttrs<'_>, Error> {
    let mut parsed_debug_attrs = ParsedDebugAttrs {
        parsed_debug_format_attr: None,
        parsed_debug_bound_attr: None,
    };

    for attr in attrs {
        match &attr.meta {
            Meta::NameValue(nv) if is_debug_attribute(&nv.path) => {
                let parsed_debug_format_attr = parse_debug_format_attr(
                    &nv.value,
                    &attr.meta,
                    provenance,
                    bound_attribute_legality,
                )?;

                if !is_debug_format_attribute_allowed(provenance) {
                    return Err(Error::new_spanned(
                        attr,
                        r#"`debug = "..."` format attribute is allowed only on struct and enum fields"#,
                    ));
                }

                if parsed_debug_attrs.parsed_debug_format_attr.is_some() {
                    return Err(Error::new_spanned(
                        attr,
                        format!(
                            r#"multiple `debug = "..."` format attributes on the same {}"#,
                            get_attr_owner_from_attrs_provenance(provenance)
                        ),
                    ));
                }

                parsed_debug_attrs.parsed_debug_format_attr = Some(parsed_debug_format_attr);
            }
            Meta::List(list) if is_debug_attribute(&list.path) => {
                let parsed_debug_bound_attr =
                    parse_debug_bound_attr(list, &attr.meta, provenance, bound_attribute_legality)?;

                match bound_attribute_legality {
                    DebugBoundAttributeLegality::Legal => {
                        if parsed_debug_attrs.parsed_debug_bound_attr.is_some() {
                            return Err(Error::new_spanned(
                                attr,
                                format!(
                                    r#"multiple `debug(bound = "...")` bound attributes on the same {}"#,
                                    get_attr_owner_from_attrs_provenance(provenance)
                                ),
                            ));
                        }

                        parsed_debug_attrs.parsed_debug_bound_attr = Some(parsed_debug_bound_attr);
                    }
                    DebugBoundAttributeLegality::IllegalAlreadySpecifiedOnStruct => {
                        return Err(Error::new_spanned(
                            attr,
                            r#"`debug(bound = "...")` bound attribute is not allowed on struct fields if already specified on the struct itself"#,
                        ))
                    }
                    DebugBoundAttributeLegality::IllegalAlreadySpecifiedOnStructFieldType => {
                        return Err(Error::new_spanned(
                            attr,
                            r#"`debug(bound = "...")` bound attribute was already specified on this struct field type"#,
                        ))
                    }
                    DebugBoundAttributeLegality::IllegalAlreadySpecifiedOnEnum => {
                        return Err(Error::new_spanned(
                            attr,
                            r#"`debug(bound = "...")` bound attribute is not allowed on enum fields if already specified on the enum itself"#,
                        ))
                    }
                    DebugBoundAttributeLegality::IllegalNotAllowedOnEnumVariant => {
                        return Err(Error::new_spanned(
                            attr,
                            r#"`debug(bound = "...")` bound attribute is not allowed on enum variants"#,
                        ))
                    }
                    DebugBoundAttributeLegality::IllegalAlreadySpecifiedOnEnumFieldType => {
                        return Err(Error::new_spanned(
                            attr,
                            r#"`debug(bound = "...")` bound attribute was already specified on this enum field type"#,
                        ))
                    }
                }
            }
            Meta::Path(p) if is_debug_attribute(p) => {
                return Err(Error::new(
                    p.span(),
                    get_invalid_debug_attr_err_msg(provenance, bound_attribute_legality),
                ))
            }
            _ => {}
        }
    }

    Ok(parsed_debug_attrs)
}

fn is_debug_attribute(path: &Path) -> bool {
    path.is_ident("debug")
}

fn is_debug_format_attribute_allowed(attrs_provenance: AttrsProvenance) -> bool {
    attrs_provenance == AttrsProvenance::StructField
        || attrs_provenance == AttrsProvenance::EnumField
}

fn is_debug_bound_attribute_allowed(bound_attribute_legality: DebugBoundAttributeLegality) -> bool {
    bound_attribute_legality == DebugBoundAttributeLegality::Legal
}

fn parse_debug_format_attr<'a>(
    expr: &'a Expr,
    attr_meta: &Meta,
    attrs_provenance: AttrsProvenance,
    bound_attribute_legality: DebugBoundAttributeLegality,
) -> Result<ParsedDebugFormatAttr<'a>, Error> {
    Ok(expr)
        .and_then(|expr| {
            if let Expr::Lit(expr_lit) = expr {
                Ok(expr_lit)
            } else {
                Err(Error::new_spanned(
                    attr_meta,
                    get_invalid_debug_attr_err_msg(attrs_provenance, bound_attribute_legality),
                ))
            }
        })
        .and_then(|expr_lit| {
            if let Lit::Str(format_str) = &expr_lit.lit {
                Ok(ParsedDebugFormatAttr { format_str })
            } else {
                Err(Error::new_spanned(
                    attr_meta,
                    get_invalid_debug_attr_err_msg(attrs_provenance, bound_attribute_legality),
                ))
            }
        })
}

fn parse_debug_bound_attr(
    list: &MetaList,
    attr_meta: &Meta,
    attrs_provenance: AttrsProvenance,
    bound_attribute_legality: DebugBoundAttributeLegality,
) -> Result<ParsedDebugBoundAttr, Error> {
    let mut bound = None;

    list.parse_nested_meta(|nested_meta| {
        if nested_meta.path.is_ident("bound") {
            bound = Some(ParsedDebugBoundAttr {
                where_predicates: nested_meta
                    .value()?
                    .parse::<LitStr>()?
                    .parse_with(Punctuated::<WherePredicate, Comma>::parse_terminated)?,
            });

            Ok(())
        } else {
            Err(Error::new_spanned(
                attr_meta,
                get_invalid_debug_attr_err_msg(attrs_provenance, bound_attribute_legality),
            ))
        }
    })?;

    bound.ok_or(Error::new_spanned(
        attr_meta,
        get_invalid_debug_attr_err_msg(attrs_provenance, bound_attribute_legality),
    ))
}

fn get_attr_owner_from_attrs_provenance(provenance: AttrsProvenance) -> &'static str {
    match provenance {
        AttrsProvenance::Struct => "struct",
        AttrsProvenance::StructField => "struct field",
        AttrsProvenance::Enum => "enum",
        AttrsProvenance::EnumField => "enum field",
        AttrsProvenance::EnumVariant => "enum variant",
    }
}

fn get_invalid_debug_attr_err_msg(
    attrs_provenance: AttrsProvenance,
    bound_attribute_legality: DebugBoundAttributeLegality,
) -> &'static str {
    let is_format_attribute_allowed = is_debug_format_attribute_allowed(attrs_provenance);
    let is_bound_attribute_allowed = is_debug_bound_attribute_allowed(bound_attribute_legality);

    if is_format_attribute_allowed && is_bound_attribute_allowed {
        r#"expected either `debug = "..."` format attribute or `debug(bound = "...")` bound attribute"#
    } else if is_format_attribute_allowed {
        r#"expected `debug = "..."` format attribute"#
    } else if is_bound_attribute_allowed {
        r#"expected `debug(bound = "...")` bound attribute"#
    } else {
        r#"`debug = "..."` format attribute and `debug(bound = "...")` bound attribute are not allowed here"#
    }
}

fn add_where_predicates_to_set(
    where_predicates: Punctuated<WherePredicate, Comma>,
    set: &mut HashSet<WherePredicate>,
) {
    for predicate in where_predicates {
        match predicate {
            WherePredicate::Lifetime(l) => {
                for bound in l.bounds {
                    let mut bounds = Punctuated::new();
                    bounds.push(bound);

                    set.insert(
                        PredicateLifetime {
                            lifetime: l.lifetime.clone(),
                            colon_token: l.colon_token,
                            bounds,
                        }
                        .into(),
                    );
                }
            }
            WherePredicate::Type(t) => {
                for bound in t.bounds {
                    let mut bounds = Punctuated::new();
                    bounds.push(bound);

                    set.insert(
                        PredicateType {
                            lifetimes: t.lifetimes.clone(),
                            bounded_ty: t.bounded_ty.clone(),
                            colon_token: t.colon_token,
                            bounds,
                        }
                        .into(),
                    );
                }
            }
            _ => {}
        }
    }
}

fn convert_struct_input_to_output(
    struct_input: &StructInputData,
    mut kind_agnostic_input: KindAgnosticInputData,
) -> Result<TokenStream, Error> {
    let fields_data = get_struct_fields_data(
        struct_input,
        kind_agnostic_input.is_where_predicates_adjustment_needed,
    )?;

    if kind_agnostic_input.is_where_predicates_adjustment_needed {
        for (field_ty, parsed_debug_bound_attr) in fields_data.types_data {
            adjust_where_predicates_based_on_field_ty(
                field_ty,
                parsed_debug_bound_attr,
                &mut kind_agnostic_input.where_predicates,
                &kind_agnostic_input.generic_ty_idents,
            );
        }
    }

    let (impl_generics, ty_generics, where_clause) = get_generics_split_for_impl(
        &mut kind_agnostic_input.generics,
        kind_agnostic_input.where_predicates,
    );

    let caller_ty = &kind_agnostic_input.caller_ty;
    let debug_fields = fields_data.debug_fields;

    let debug_helper = if fields_data.is_tuple_struct {
        quote! { debug_tuple }
    } else {
        quote! { debug_struct }
    };

    let output = quote! {
        impl #impl_generics ::core::fmt::Debug for #caller_ty #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.#debug_helper(::core::stringify!(#caller_ty))
                    #(#debug_fields)*
                    .finish()
            }
        }
    };

    Ok(output.into())
}

type TypesData<'a> = HashMap<&'a Type, Option<ParsedDebugBoundAttr>>;

struct StructFieldsData<'a, T: ToTokens> {
    debug_fields: Vec<T>,
    types_data: TypesData<'a>,
    is_tuple_struct: bool,
}

fn get_struct_fields_data(
    struct_input: &StructInputData,
    is_where_predicates_adjustment_needed: bool,
) -> Result<StructFieldsData<'_, impl ToTokens>, Error> {
    let mut types_data = TypesData::new();
    let mut is_tuple_struct = false;

    let default_debug_bound_attribute_legality = if is_where_predicates_adjustment_needed {
        DebugBoundAttributeLegality::Legal
    } else {
        DebugBoundAttributeLegality::IllegalAlreadySpecifiedOnStruct
    };

    let debug_fields = struct_input
        .data_struct
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let parsed_debug_format_attr = parse_field_debug_attributes(
                field,
                default_debug_bound_attribute_legality,
                DebugBoundAttributeLegality::IllegalAlreadySpecifiedOnStructFieldType,
                AttrsProvenance::StructField,
                &mut types_data,
            )?;

            if let Some(ident) = field.ident.as_ref() {
                if let Some(parsed_debug_format_attr) = parsed_debug_format_attr {
                    let format_str = parsed_debug_format_attr.format_str;
                    let format_arg = quote_spanned! { field.ty.span() => self.#ident };
                    Ok(quote! {
                        .field(::core::stringify!(#ident), &::core::format_args!(#format_str, #format_arg))
                    })
                } else {
                    let debug_arg = quote_spanned! { field.ty.span() => &self.#ident };
                    Ok(quote! {
                        .field(::core::stringify!(#ident), #debug_arg)
                    })
                }
            } else {
                is_tuple_struct = true;
                let index = Index::from(index);

                if let Some(parsed_debug_format_attr) = parsed_debug_format_attr {
                    let format_str = parsed_debug_format_attr.format_str;
                    let format_arg = quote_spanned! { field.ty.span() => self.#index };

                    Ok(quote! {
                        .field(&::core::format_args!(#format_str, #format_arg))
                    })
                } else {
                    let debug_arg = quote_spanned! { field.ty.span() => &self.#index };

                    Ok(quote! {
                        .field(#debug_arg)
                    })
                }
            }
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(StructFieldsData {
        debug_fields,
        types_data,
        is_tuple_struct,
    })
}

fn convert_enum_input_to_output(
    enum_input: &EnumInputData,
    mut kind_agnostic_input: KindAgnosticInputData,
) -> Result<TokenStream, Error> {
    let caller_ty = &kind_agnostic_input.caller_ty;

    let enum_fields_data = get_enum_fields_data(
        enum_input,
        caller_ty,
        kind_agnostic_input.is_where_predicates_adjustment_needed,
    )?;

    if kind_agnostic_input.is_where_predicates_adjustment_needed {
        for (field_ty, parsed_debug_bound_attr) in enum_fields_data.types_data {
            adjust_where_predicates_based_on_field_ty(
                field_ty,
                parsed_debug_bound_attr,
                &mut kind_agnostic_input.where_predicates,
                &kind_agnostic_input.generic_ty_idents,
            );
        }
    }

    let (impl_generics, ty_generics, where_clause) = get_generics_split_for_impl(
        &mut kind_agnostic_input.generics,
        kind_agnostic_input.where_predicates,
    );

    let match_arms = enum_fields_data.match_arms;

    let output = if match_arms.is_empty() {
        quote! {
            impl #impl_generics ::core::fmt::Debug for #caller_ty #ty_generics #where_clause {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.write_str(::core::stringify!(#caller_ty))
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics ::core::fmt::Debug for #caller_ty #ty_generics #where_clause {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        #(#match_arms)*
                    }
                }
            }
        }
    };

    Ok(output.into())
}

struct EnumFieldsData<'a, T: ToTokens> {
    match_arms: Vec<T>,
    types_data: TypesData<'a>,
}

fn get_enum_fields_data<'a>(
    enum_input: &'a EnumInputData,
    caller_ty: &Ident,
    is_where_predicates_adjustment_needed: bool,
) -> Result<EnumFieldsData<'a, impl ToTokens>, Error> {
    let mut types_data = TypesData::new();

    let default_debug_bound_attribute_legality = if is_where_predicates_adjustment_needed {
        DebugBoundAttributeLegality::Legal
    } else {
        DebugBoundAttributeLegality::IllegalAlreadySpecifiedOnEnum
    };

    let match_arms = enum_input
        .data_enum
        .variants
        .iter()
        .map(|variant| {
            parse_debug_attrs(
                &variant.attrs,
                AttrsProvenance::EnumVariant,
                DebugBoundAttributeLegality::IllegalNotAllowedOnEnumVariant,
            )?;

            let field_idents_and_formatting_operations = variant
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let parsed_debug_format_attr = parse_field_debug_attributes(
                        field,
                        default_debug_bound_attribute_legality,
                        DebugBoundAttributeLegality::IllegalAlreadySpecifiedOnEnumFieldType,
                        AttrsProvenance::EnumField,
                        &mut types_data,
                    )?;

                    let field_ident = format_ident!("m{}", Index::from(index));
                    let field_ident_arg = quote_spanned! { field.ty.span() => #field_ident };

                    if let Some(parsed_debug_format_attr) = parsed_debug_format_attr {
                        let format_str = parsed_debug_format_attr.format_str;

                        Ok((
                            quote! { #field_ident },
                            quote! {
                                .field(&::core::format_args!(#format_str, #field_ident_arg))
                            },
                        ))
                    } else {
                        Ok((
                            quote! { #field_ident },
                            quote! {
                                .field(&#field_ident_arg)
                            },
                        ))
                    }
                })
                .collect::<Result<Vec<_>, Error>>()?;

            let variant_ident = &variant.ident;

            let field_idents = field_idents_and_formatting_operations
                .iter()
                .map(|(field_ident, _)| field_ident);

            let formatting_operations = field_idents_and_formatting_operations
                .iter()
                .map(|(_, formatting_operation)| formatting_operation);

            if field_idents_and_formatting_operations.is_empty() {
                Ok(quote! {
                    #caller_ty::#variant_ident => f.write_str(::core::stringify!(#variant_ident)),
                })
            } else {
                Ok(quote! {
                    #caller_ty::#variant_ident(#(#field_idents),*) => {
                        f.debug_tuple(::core::stringify!(#variant_ident))
                            #(#formatting_operations)*
                            .finish()
                    },
                })
            }
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(EnumFieldsData {
        match_arms,
        types_data,
    })
}

fn parse_field_debug_attributes<'a>(
    field: &'a Field,
    default_debug_bound_attribute_legality: DebugBoundAttributeLegality,
    debug_bound_legality_when_already_specified_on_type: DebugBoundAttributeLegality,
    attrs_provenance: AttrsProvenance,
    types_data: &mut TypesData<'a>,
) -> Result<Option<ParsedDebugFormatAttr<'a>>, Error> {
    let parsed_debug_format_attr = match types_data.entry(&field.ty) {
        Entry::Occupied(mut parsed_debug_bound_attr_entry) => {
            let debug_bound_attribute_legality = if parsed_debug_bound_attr_entry.get().is_some() {
                debug_bound_legality_when_already_specified_on_type
            } else {
                default_debug_bound_attribute_legality
            };

            let parsed_debug_attrs = parse_debug_attrs(
                &field.attrs,
                attrs_provenance,
                debug_bound_attribute_legality,
            )?;

            *parsed_debug_bound_attr_entry.get_mut() = parsed_debug_attrs.parsed_debug_bound_attr;

            parsed_debug_attrs.parsed_debug_format_attr
        }
        Entry::Vacant(entry) => {
            let parsed_debug_attrs = parse_debug_attrs(
                &field.attrs,
                attrs_provenance,
                default_debug_bound_attribute_legality,
            )?;

            entry.insert(parsed_debug_attrs.parsed_debug_bound_attr);

            parsed_debug_attrs.parsed_debug_format_attr
        }
    };

    Ok(parsed_debug_format_attr)
}

fn get_generics_split_for_impl(
    generics: &mut Generics,
    where_predicates: HashSet<WherePredicate>,
) -> (ImplGenerics<'_>, TypeGenerics<'_>, Option<&WhereClause>) {
    if !where_predicates.is_empty() {
        generics
            .make_where_clause()
            .predicates
            .extend(where_predicates);
    }

    generics.split_for_impl()
}

fn adjust_where_predicates_based_on_field_ty(
    field_ty: &Type,
    parsed_debug_bound_attr: Option<ParsedDebugBoundAttr>,
    where_predicates: &mut HashSet<WherePredicate>,
    generic_ty_idents: &HashSet<Ident>,
) {
    if let Some(parsed_debug_bound_attr) = parsed_debug_bound_attr {
        add_where_predicates_to_set(parsed_debug_bound_attr.where_predicates, where_predicates);
        return;
    }

    let mut stack = vec![field_ty];
    let mut ty_needed_to_impl_debug = None;
    let mut is_in_phantom_data_ty = false;

    while let Some(current_ty) = stack.pop() {
        match current_ty {
            Type::Array(t) => stack.push(&t.elem),
            Type::Group(t) => stack.push(&t.elem),
            Type::Paren(t) => stack.push(&t.elem),
            Type::Path(t) => {
                if let Some(qself) = t.qself.as_ref() {
                    ty_needed_to_impl_debug = ty_needed_to_impl_debug.or(Some(t));
                    stack.push(qself.ty.as_ref());
                } else {
                    let path = &t.path;

                    if let Some(phantom_data_generic_ty) = get_phantom_data_generic_ty(path) {
                        if ty_needed_to_impl_debug.is_some() {
                            is_in_phantom_data_ty = true;
                            stack.push(phantom_data_generic_ty);
                        }
                    } else {
                        if let Some(first_segment) = path.segments.first() {
                            if first_segment.arguments.is_none()
                                && path.segments.len() > 1
                                && generic_ty_idents.contains(&first_segment.ident)
                            {
                                if let Some(ty) = ty_needed_to_impl_debug {
                                    where_predicates.insert(parse_quote!(#ty: ::core::fmt::Debug));
                                } else if !is_in_phantom_data_ty {
                                    where_predicates
                                        .insert(parse_quote!(#path: ::core::fmt::Debug));
                                }

                                continue;
                            }
                        }

                        if let Some(last_segment) = path.segments.last() {
                            match &last_segment.arguments {
                                PathArguments::None => {
                                    let ident = &last_segment.ident;

                                    if generic_ty_idents.contains(ident) {
                                        if let Some(ty) = ty_needed_to_impl_debug {
                                            where_predicates
                                                .insert(parse_quote!(#ty: ::core::fmt::Debug));
                                        } else if !is_in_phantom_data_ty {
                                            where_predicates
                                                .insert(parse_quote!(#ident: ::core::fmt::Debug));
                                        }
                                    }
                                }
                                PathArguments::AngleBracketed(a) => {
                                    a.args
                                        .iter()
                                        .filter_map(|arg| match arg {
                                            GenericArgument::Type(t) => Some(t),
                                            _ => None,
                                        })
                                        .for_each(|t| stack.push(t));
                                }
                                PathArguments::Parenthesized(_) => {}
                            }
                        }
                    }
                }
            }
            Type::Reference(t) => stack.push(&t.elem),
            Type::Slice(t) => stack.push(&t.elem),
            Type::Tuple(t) => t.elems.iter().for_each(|t| stack.push(t)),
            _ => {}
        }
    }
}

fn get_phantom_data_generic_ty(path: &Path) -> Option<&Type> {
    let mut segments = path.segments.iter();

    match (
        segments.next(),
        segments.next(),
        segments.next(),
        segments.next(),
    ) {
        (Some(first_segment), None, None, None) if path.leading_colon.is_none() => {
            get_phantom_data_path_segment_generic_ty(first_segment)
        }
        (Some(first_segment), Some(second_segment), None, None)
            if path.leading_colon.is_none() && is_marker_path_segment(first_segment) =>
        {
            get_phantom_data_path_segment_generic_ty(second_segment)
        }
        (Some(first_segment), Some(second_segment), Some(third_segment), None)
            if is_std_path_segment(first_segment) && is_marker_path_segment(second_segment) =>
        {
            get_phantom_data_path_segment_generic_ty(third_segment)
        }
        _ => None,
    }
}

fn is_std_path_segment(segment: &PathSegment) -> bool {
    segment.ident == "std" && segment.arguments.is_none()
}

fn is_marker_path_segment(segment: &PathSegment) -> bool {
    segment.ident == "marker" && segment.arguments.is_none()
}

fn get_phantom_data_path_segment_generic_ty(segment: &PathSegment) -> Option<&Type> {
    (segment.ident == "PhantomData")
        .then(|| match &segment.arguments {
            PathArguments::AngleBracketed(a) => {
                let mut args = a.args.iter();

                match (args.next(), args.next()) {
                    (Some(GenericArgument::Type(t)), None) => Some(t),
                    _ => None,
                }
            }
            _ => None,
        })
        .flatten()
}
