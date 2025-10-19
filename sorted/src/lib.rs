//! Provides the [`#[sorted]`](macro@sorted) and [`#[check]`](macro@check) attribute macros.

mod sortable_items;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use sortable_items::{SortableItem, SortableItemValueEnum, SortableItems, StringSegments};
use syn::{
    parse_macro_input,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Arm, Error, ExprMatch, Item, ItemFn, Lit, Pat, PatIdent, PatLit, PatOr, PatParen, PatPath,
    PatStruct, PatTupleStruct, Path,
};

/// An attribute macro that ensures that enum variants and match arms stay in sorted order.
/// The macro will detect unsorted items at compile time and emit
/// an error pointing out which items are out of order.
///
/// # Ensuring that enum variants are sorted
///
/// Simply put the [`#[sorted]`](macro@sorted) attribute macro on the enum declaration and
/// you will get a compile error if the variants are out of order:
///
/// ```compile_fail
/// use sorted::sorted;
///
/// #[sorted]
/// enum Foo {
///     A,
///     C,
///     B,
/// }
/// ```
///
/// # Ensuring that match arms are sorted
///
/// Put the [`#[sorted]`](macro@sorted) attribute macro on the match expression that
/// you want to check. You will also need to put the [`#[sorted::check]`](macro@check)
/// attribute macro on the function that contains the match expression due to a limitation
/// in the stable compiler that does not allow procedural macro invocations on expressions:
///
/// ```compile_fail
/// #[allow(dead_code)]
/// #[sorted::check]
/// fn example() {
///     let x = 0;
///
///     #[sorted]
///     match x {
///         1 => println!("one"),
///         0 => println!("zero"),
///         _ => println!("other"),
///     }
/// }
/// ```
///
/// You can apply the macro on match arms consisting of:
/// - literals:
/// ```
/// #[allow(dead_code)]
/// #[sorted::check]
/// fn example() {
///     let x = "abc";
///
///     #[sorted]
///     match x {
///         "abc" => println!("abc"),
///         "def" => println!("def"),
///         _ => println!("other"),
///     }
/// }
/// ```
/// - identifiers (note that the `ref [mut] <identifier>` part
///   is ignored, only the part after the `@` symbol is checked):
/// ```
/// #[allow(dead_code)]
/// #[sorted::check]
/// fn example() {
///     let mut x = "abc";
///
///     #[sorted]
///     match x {
///         ref mut a @ "abc" => println!("{a}"),
///         ref mut b @ "def" => println!("{b}"),
///         _ => println!("other"),
///     }
/// }
/// ```
/// - or-patterns (note that the or-pattern itself is compared with
///   other patterns only based on the value of its first sub-pattern):
/// ```
/// #[allow(dead_code)]
/// #[sorted::check]
/// fn example() {
///     let x = 0;
///
///     #[sorted]
///     match x {
///         0 | 5 => println!("0 or 5"),
///         1 | 2 => println!("1 or 2"),
///         _ => println!("other"),
///     }
/// }
/// ```
/// - parenthesized patterns:
/// ```
/// #[allow(dead_code, unused_parens)]
/// #[sorted::check]
/// fn example() {
///     let x = 0;
///
///     #[sorted]
///     match x {
///         0 | 5 => println!("0 or 5"),
///         (1 | 2) | (3 | 4) => println!("1, 2, 3, or 4"),
///         _ => println!("other"),
///     }
/// }
/// ```
/// - paths without self-qualifiers (note that any path arguments are ignored):
/// ```
/// use std::marker::PhantomData;
///
/// #[allow(dead_code)]
/// #[sorted::check]
/// fn example() {
///     let x = 0;
///
///     #[sorted]
///     match x {
///         Struct::<B>::MAX => println!("{}", Struct::<B>::MAX),
///         Struct::<A>::MIN => println!("{}", Struct::<A>::MIN),
///         _ => println!("other"),
///     }
/// }
///
/// trait Range {
///     const MIN: u32;
///     const MAX: u32;
/// }
///
/// struct Struct<T> {
///     dummy: PhantomData<T>
/// }
///
/// struct A;
///
/// impl Range for Struct<A> {
///     const MIN: u32 = 0;
///     const MAX: u32 = 5;
/// }
///
/// struct B;
///
/// impl Range for Struct<B> {
///     const MIN: u32 = 10;
///     const MAX: u32 = 15;
/// }
/// ```
/// - structs without self-qualifiers (note that or-patterns inside the struct are checked
///   but everything else is ignored):
/// ```
/// #[allow(dead_code)]
/// #[sorted::check]
/// fn example() {
///     let x = Container{ value: 0 };
///
///     #[sorted]
///     match x {
///         Container{ value: 1 | 2 | 3 } => println!("one, two or three"),
///         Container{ value: 0 } => println!("zero"),
///         _ => println!("other"),
///     }
/// }
///
/// struct Container {
///     value: u32,
/// }
/// ```
/// - tuple structs without self-qualifiers (note that or-patterns inside the tuple struct
///   are checked but everything else is ignored):
/// ```
/// #[allow(dead_code)]
/// #[sorted::check]
/// fn example() {
///     let x = Container(0);
///
///     #[sorted]
///     match x {
///         Container(1 | 2 | 3) => println!("one, two or three"),
///         Container(0) => println!("zero"),
///         _ => println!("other"),
///     }
/// }
///
/// struct Container(u32);
/// ```
///
/// Other kinds of patterns, or mismatches between them, are not supported and
/// will cause a compile error if the macro is applied on a match expression containing them.
#[proc_macro_attribute]
pub fn sorted(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = args;

    let original_input = input.clone();
    let item = parse_macro_input!(input as Item);

    match look_for_unsorted_enum_variants(item) {
        Ok(()) => original_input,
        Err(e) => {
            let mut output = TokenStream::from(original_input);
            output.extend(e.to_compile_error());
            output.into()
        }
    }
}

/// An attribute macro that works around the current limitation of
/// the stable compiler that does not allow procedural macro invocations
/// on expressions.
///
/// This macro will expand by looking inside the function on which it is placed
/// to find any match expressions carrying a [`#[sorted]`](macro@sorted) attribute, checking the order of
/// the arms in that match expression, and then stripping away the inner
/// [`#[sorted]`](macro@sorted) attribute to prevent the stable compiler from refusing to compile
/// the code.
///
/// For example, the following code will not compile on the current stable compiler:
///
/// ```compile_fail
/// fn example() {
///     let x = 0;
///
///     #[sorted]
///     match x {
///         0 => println!("zero"),
///         1 => println!("one"),
///         _ => println!("other"),
///     }
/// }
/// ```
///
/// But add the [`#[sorted::check]`](macro@check) attribute macro and it will compile:
///
/// ```
/// #[sorted::check] // <-- added here
/// fn example() {
///     let x = 0;
///
///     #[sorted]
///     match x {
///         0 => println!("zero"),
///         1 => println!("one"),
///         _ => println!("other"),
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn check(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = args;
    let item = parse_macro_input!(input as ItemFn);

    look_for_unsorted_match_patterns(item).into()
}

fn look_for_unsorted_enum_variants(input: Item) -> Result<(), Error> {
    let Item::Enum(input_enum) = input else {
        return Err(Error::new(
            Span::call_site(),
            "expected enum or match expression",
        ));
    };

    let sortable_items: Vec<SortableItem<StringSegments>> = input_enum
        .variants
        .into_iter()
        .map(|v| v.ident.into())
        .collect();

    SortableItems::StringSegments(sortable_items).look_for_unsorted_items()
}

fn look_for_unsorted_match_patterns(mut input: ItemFn) -> TokenStream {
    let mut visitor = Visitor { error: None };
    visitor.visit_item_fn_mut(&mut input);

    let mut output = input.into_token_stream();

    if let Some(e) = visitor.error {
        output.extend(e.into_compile_error());
    }

    output
}

struct Visitor {
    error: Option<Error>,
}

impl VisitMut for Visitor {
    fn visit_expr_match_mut(&mut self, match_expression: &mut ExprMatch) {
        let mut found_sorted_attribute = false;

        match_expression.attrs.retain(|a| {
            if is_sorted_attribute(a.path()) {
                found_sorted_attribute = true;
                false
            } else {
                true
            }
        });

        if found_sorted_attribute {
            self.error = look_for_unsorted_match_arms(&match_expression.arms).err();
            if self.error.is_some() {
                return;
            }
        }

        visit_mut::visit_expr_match_mut(self, match_expression);
    }
}

fn is_sorted_attribute(path: &Path) -> bool {
    let segments_len = path.segments.len();
    (segments_len == 1 || segments_len == 2) && path.segments.iter().all(|s| s.ident == "sorted")
}

fn look_for_unsorted_match_arms(arms: &[Arm]) -> Result<(), Error> {
    let mut sortable_items_in_match_arms = SortableItems::new_speculative();
    let mut sortable_items_in_or_patterns = Vec::new();
    let mut storage_stack: Vec<FrameStorage> = Vec::new();

    let mut stack = vec![FrameOperation::ParseArm(ParseArmOperation { arm_index: 0 })];

    while let Some(operation) = stack.pop() {
        match operation {
            FrameOperation::AppendSortableItemInMatchArms => {
                if let Some(storage) = storage_stack.pop() {
                    sortable_items_in_match_arms.try_append(storage.value, storage.token_stream)?;
                }
            }
            FrameOperation::AppendSortableItemInOrPatterns(op) => {
                let mut sortable_items =
                    SortableItems::new_speculative_with_capacity(op.count_of_or_patterns);

                for _ in 0..op.count_of_or_patterns {
                    if let Some(storage) = storage_stack.pop() {
                        sortable_items.try_append(storage.value, storage.token_stream)?;
                    } else {
                        break;
                    }
                }

                sortable_items_in_or_patterns.push(sortable_items);
            }
            FrameOperation::AppendSortableItemInOrPatternsAndCorrectStorageStack(op) => {
                let mut sortable_items =
                    SortableItems::new_speculative_with_capacity(op.count_of_or_patterns);

                let Some(first_sortable_item) = storage_stack.pop() else {
                    break;
                };

                sortable_items.try_append(
                    first_sortable_item.value.clone(),
                    first_sortable_item.token_stream,
                )?;

                for _ in 0..op.count_of_or_patterns - 1 {
                    if let Some(storage) = storage_stack.pop() {
                        sortable_items.try_append(storage.value, storage.token_stream)?;
                    } else {
                        break;
                    }
                }

                sortable_items_in_or_patterns.push(sortable_items);

                /*
                    When we have an OR pattern, we compare it with other
                    patterns only based on the first item in the OR pattern.
                */
                storage_stack[op.storage_index_to_correct].value = first_sortable_item.value;
            }
            FrameOperation::ParseArm(op) => {
                if op.arm_index >= arms.len() {
                    break;
                }

                stack.push(FrameOperation::ParseArm(ParseArmOperation {
                    arm_index: op.arm_index + 1,
                }));

                stack.push(FrameOperation::AppendSortableItemInMatchArms);

                let arm = &arms[op.arm_index];

                if let Some(guard) = &arm.guard {
                    return Err(Error::new(
                        guard.0.span.join(guard.1.span()).unwrap_or(guard.0.span),
                        "match arms with guards are unsupported by #[sorted]",
                    ));
                }

                storage_stack.push(FrameStorage {
                    value: SortableItemValueEnum::new_speculative(),
                    token_stream: arm.pat.to_token_stream(),
                });

                let storage_stack_len = storage_stack.len();
                parse_pattern(
                    &arm.pat,
                    &mut stack,
                    &mut storage_stack,
                    storage_stack_len - 1,
                    None,
                )?;
            }
            FrameOperation::ParseSubPattern(op) => {
                parse_pattern(
                    op.sub_pattern,
                    &mut stack,
                    &mut storage_stack,
                    op.storage_index,
                    op.storage_index_to_correct,
                )?;
            }
            FrameOperation::LookOnlyForUnsortedOrSubPatterns(op) => {
                look_only_for_unsorted_or_sub_patterns(
                    op.sub_pattern,
                    &mut stack,
                    &mut storage_stack,
                );
            }
        }
    }

    for sortable_items in &sortable_items_in_or_patterns {
        sortable_items.look_for_unsorted_items()?;
    }

    sortable_items_in_match_arms.look_for_unsorted_items()
}

fn parse_pattern<'a>(
    pat: &'a Pat,
    stack: &mut Vec<FrameOperation<'a>>,
    storage_stack: &mut Vec<FrameStorage>,
    storage_index: usize,
    storage_index_to_correct: Option<usize>,
) -> Result<(), Error> {
    let current_storage = &mut storage_stack[storage_index];

    match pat {
        Pat::Ident(pi) => {
            parse_ident_pattern(pi, stack, current_storage, storage_index)?;
        }
        Pat::Lit(pl) => {
            parse_lit_pattern(pl, current_storage)?;
        }
        Pat::Or(po) => {
            parse_or_pattern(po, stack, storage_stack, storage_index_to_correct);
        }
        Pat::Paren(pp) => {
            parse_paren_pattern(pp, stack, storage_index, storage_index_to_correct);
        }
        Pat::Path(pp) => {
            parse_path_pattern(pp, current_storage)?;
        }
        Pat::Struct(ps) => {
            parse_struct_pattern(ps, stack, current_storage)?;
        }
        Pat::TupleStruct(pts) => {
            parse_tuple_struct_pattern(pts, stack, current_storage)?;
        }
        Pat::Wild(_) => {
            parse_wildcard_pattern(current_storage);
        }
        _ => {
            return Err(generate_generic_unsupported_error(
                &current_storage.token_stream,
            ));
        }
    }

    Ok(())
}

fn parse_ident_pattern<'a>(
    pat: &'a PatIdent,
    stack: &mut Vec<FrameOperation<'a>>,
    current_storage: &mut FrameStorage,
    storage_index: usize,
) -> Result<(), Error> {
    if let Some(subpat) = &pat.subpat {
        stack.push(FrameOperation::ParseSubPattern(ParseSubPatternOperation {
            sub_pattern: &subpat.1,
            storage_index,
            storage_index_to_correct: None,
        }));
    } else if pat.by_ref.is_none() && pat.mutability.is_none() {
        current_storage.value = (&pat.ident).into();
    } else {
        return Err(Error::new_spanned(
            &current_storage.token_stream,
            "`ref [mut]` patterns that bind variables without `@ <subpattern>` are unsupported by #[sorted]",
        ));
    }

    Ok(())
}

fn parse_lit_pattern(pat: &PatLit, current_storage: &mut FrameStorage) -> Result<(), Error> {
    match &pat.lit {
        Lit::Str(s) => {
            current_storage.value = s.into();
        }
        Lit::ByteStr(s) => {
            current_storage.value = s.into();
        }
        Lit::CStr(s) => {
            current_storage.value = s.into();
        }
        Lit::Byte(b) => {
            current_storage.value = b.into();
        }
        Lit::Char(c) => {
            current_storage.value = c.into();
        }
        Lit::Int(i) => {
            current_storage.value = i
                .try_into()
                .map_err(|_| generate_generic_unsupported_error(&current_storage.token_stream))?;
        }
        Lit::Float(f) => {
            current_storage.value = f
                .try_into()
                .map_err(|_| generate_generic_unsupported_error(&current_storage.token_stream))?;
        }
        Lit::Bool(b) => {
            current_storage.value = b.into();
        }
        _ => {
            return Err(generate_generic_unsupported_error(
                &current_storage.token_stream,
            ));
        }
    }

    Ok(())
}

fn parse_or_pattern<'a>(
    pat: &'a PatOr,
    stack: &mut Vec<FrameOperation<'a>>,
    storage_stack: &mut Vec<FrameStorage>,
    storage_index_to_correct: Option<usize>,
) {
    stack.push(
        FrameOperation::AppendSortableItemInOrPatternsAndCorrectStorageStack(
            AppendSortableItemInOrPatternsAndCorrectStorageStackOperation {
                count_of_or_patterns: pat.cases.len(),
                storage_index_to_correct: storage_index_to_correct
                    .unwrap_or(storage_stack.len() - 1),
            },
        ),
    );

    for case in pat.cases.iter().rev() {
        storage_stack.push(FrameStorage {
            value: SortableItemValueEnum::new_speculative(),
            token_stream: case.to_token_stream(),
        });

        stack.push(FrameOperation::ParseSubPattern(ParseSubPatternOperation {
            sub_pattern: case,
            storage_index: storage_stack.len() - 1,
            storage_index_to_correct: Some(storage_stack.len() - 1),
        }));
    }
}

fn parse_paren_pattern<'a>(
    pat: &'a PatParen,
    stack: &mut Vec<FrameOperation<'a>>,
    storage_index: usize,
    storage_index_to_correct: Option<usize>,
) {
    stack.push(FrameOperation::ParseSubPattern(ParseSubPatternOperation {
        sub_pattern: &pat.pat,
        storage_index,
        storage_index_to_correct,
    }));
}

fn parse_path_pattern(pat: &PatPath, current_storage: &mut FrameStorage) -> Result<(), Error> {
    if pat.qself.is_some() {
        return Err(generate_qself_unsupported_error(
            &current_storage.token_stream,
        ));
    }

    current_storage.value = (&pat.path).into();
    Ok(())
}

fn parse_struct_pattern<'a>(
    pat: &'a PatStruct,
    stack: &mut Vec<FrameOperation<'a>>,
    current_storage: &mut FrameStorage,
) -> Result<(), Error> {
    if pat.qself.is_some() {
        return Err(generate_qself_unsupported_error(
            &current_storage.token_stream,
        ));
    }

    current_storage.value = (&pat.path).into();

    for field in pat.fields.iter().rev() {
        stack.push(FrameOperation::LookOnlyForUnsortedOrSubPatterns(
            LookOnlyForUnsortedOrSubPatternsOperation {
                sub_pattern: &field.pat,
            },
        ));
    }

    Ok(())
}

fn parse_tuple_struct_pattern<'a>(
    pat: &'a PatTupleStruct,
    stack: &mut Vec<FrameOperation<'a>>,
    current_storage: &mut FrameStorage,
) -> Result<(), Error> {
    if pat.qself.is_some() {
        return Err(generate_qself_unsupported_error(
            &current_storage.token_stream,
        ));
    }

    current_storage.value = (&pat.path).into();

    for elem in pat.elems.iter().rev() {
        stack.push(FrameOperation::LookOnlyForUnsortedOrSubPatterns(
            LookOnlyForUnsortedOrSubPatternsOperation { sub_pattern: elem },
        ));
    }

    Ok(())
}

fn parse_wildcard_pattern(current_storage: &mut FrameStorage) {
    current_storage.value = SortableItemValueEnum::Wildcard;
}

fn look_only_for_unsorted_or_sub_patterns<'a>(
    pat: &'a Pat,
    stack: &mut Vec<FrameOperation<'a>>,
    storage_stack: &mut Vec<FrameStorage>,
) {
    match pat {
        Pat::Ident(pi) => {
            if let Some(subpat) = &pi.subpat {
                stack.push(FrameOperation::LookOnlyForUnsortedOrSubPatterns(
                    LookOnlyForUnsortedOrSubPatternsOperation {
                        sub_pattern: &subpat.1,
                    },
                ));
            }
        }
        Pat::Or(po) => {
            stack.push(FrameOperation::AppendSortableItemInOrPatterns(
                AppendSortableItemInOrPatternsOperation {
                    count_of_or_patterns: po.cases.len(),
                },
            ));

            for case in po.cases.iter().rev() {
                storage_stack.push(FrameStorage {
                    value: SortableItemValueEnum::new_speculative(),
                    token_stream: case.to_token_stream(),
                });

                stack.push(FrameOperation::ParseSubPattern(ParseSubPatternOperation {
                    sub_pattern: case,
                    storage_index: storage_stack.len() - 1,
                    storage_index_to_correct: Some(storage_stack.len() - 1),
                }));
            }
        }
        Pat::Paren(pp) => {
            stack.push(FrameOperation::LookOnlyForUnsortedOrSubPatterns(
                LookOnlyForUnsortedOrSubPatternsOperation {
                    sub_pattern: &pp.pat,
                },
            ));
        }
        Pat::Reference(pr) => {
            stack.push(FrameOperation::LookOnlyForUnsortedOrSubPatterns(
                LookOnlyForUnsortedOrSubPatternsOperation {
                    sub_pattern: &pr.pat,
                },
            ));
        }
        Pat::Slice(ps) => {
            for elem in ps.elems.iter().rev() {
                stack.push(FrameOperation::LookOnlyForUnsortedOrSubPatterns(
                    LookOnlyForUnsortedOrSubPatternsOperation { sub_pattern: elem },
                ));
            }
        }
        Pat::Struct(ps) => {
            for field in ps.fields.iter().rev() {
                stack.push(FrameOperation::LookOnlyForUnsortedOrSubPatterns(
                    LookOnlyForUnsortedOrSubPatternsOperation {
                        sub_pattern: &field.pat,
                    },
                ));
            }
        }
        Pat::Tuple(pt) => {
            for elem in pt.elems.iter().rev() {
                stack.push(FrameOperation::LookOnlyForUnsortedOrSubPatterns(
                    LookOnlyForUnsortedOrSubPatternsOperation { sub_pattern: elem },
                ));
            }
        }
        Pat::TupleStruct(pts) => {
            for elem in pts.elems.iter().rev() {
                stack.push(FrameOperation::LookOnlyForUnsortedOrSubPatterns(
                    LookOnlyForUnsortedOrSubPatternsOperation { sub_pattern: elem },
                ));
            }
        }
        Pat::Type(pt) => {
            stack.push(FrameOperation::LookOnlyForUnsortedOrSubPatterns(
                LookOnlyForUnsortedOrSubPatternsOperation {
                    sub_pattern: &pt.pat,
                },
            ));
        }
        _ => {}
    }
}

enum FrameOperation<'a> {
    AppendSortableItemInMatchArms,
    AppendSortableItemInOrPatterns(AppendSortableItemInOrPatternsOperation),
    AppendSortableItemInOrPatternsAndCorrectStorageStack(
        AppendSortableItemInOrPatternsAndCorrectStorageStackOperation,
    ),
    ParseArm(ParseArmOperation),
    ParseSubPattern(ParseSubPatternOperation<'a>),
    LookOnlyForUnsortedOrSubPatterns(LookOnlyForUnsortedOrSubPatternsOperation<'a>),
}

struct AppendSortableItemInOrPatternsOperation {
    count_of_or_patterns: usize,
}

struct AppendSortableItemInOrPatternsAndCorrectStorageStackOperation {
    count_of_or_patterns: usize,
    storage_index_to_correct: usize,
}

struct ParseArmOperation {
    arm_index: usize,
}

struct ParseSubPatternOperation<'a> {
    sub_pattern: &'a Pat,
    storage_index: usize,
    storage_index_to_correct: Option<usize>,
}

struct LookOnlyForUnsortedOrSubPatternsOperation<'a> {
    sub_pattern: &'a Pat,
}

struct FrameStorage {
    value: SortableItemValueEnum,
    token_stream: TokenStream,
}

fn generate_generic_unsupported_error(token_stream: &TokenStream) -> Error {
    Error::new_spanned(token_stream, "unsupported by #[sorted]")
}

fn generate_qself_unsupported_error(token_stream: &TokenStream) -> Error {
    Error::new_spanned(token_stream, "qualified paths are unsupported by #[sorted]")
}
