//! Provides the [`seq()`] function-like macro.

mod construct;
mod parse;
mod seq_data;

use seq_data::SeqData;
use syn::{buffer::TokenBuffer, parse_macro_input};

/// A function-like macro that provides a syntax for stamping out
/// sequentially indexed copies of an arbitrary chunk of code. It
/// allows you to repeat sequences of text a specified number of
/// times and even paste the repetition index inside of the text.
///
/// # Examples
///
/// ## Repeating text
///
/// ```
/// use seq::seq;
///
/// seq!(_ in 0..4 {
///     println!("hello");
/// });
/// ```
///
/// This will print "hello" four times.
///
/// ## Creating identifiers based on the repetition index
///
/// ```
/// use seq::seq;
///  
/// seq!(N in 0..2 {
///     fn foo~N() -> u32 {
///         N
///     }
///
///     fn test~N~bar() -> u32 {
///        N * 2
///     }
///
///     fn baz~N~~N() -> u32 {
///         N * 3
///     }
/// });
///
/// assert_eq!(foo0(), 0);
/// assert_eq!(foo1(), 1);
///
/// assert_eq!(test0bar(), 0);
/// assert_eq!(test1bar(), 2);
///
/// assert_eq!(baz00(), 0);
/// assert_eq!(baz11(), 3);
/// ```
///
/// Instead of repeating the whole macro body, you can also
/// specify which sequence to repeat.
///
/// ```
/// use seq::seq;
///
/// seq!(N in 1..=4 {
///     #[repr(u8)]
///     enum Irq {
///         #(Irq~N = N,)*
///     }
/// });
///
/// assert_eq!(Irq::Irq1 as u8, 1);
/// assert_eq!(Irq::Irq2 as u8, 2);
/// assert_eq!(Irq::Irq3 as u8, 3);
/// assert_eq!(Irq::Irq4 as u8, 4);
/// ```
///
/// You can combine the repetition strategies and even nest
/// them to generate complex code.
///
/// ```
/// use seq::seq;
///
/// seq!(N in 0..=1 {
///     #[repr(u8)]
///     enum RepeatedEnum0 {
///         #(Variant~N,)*
///     }
///
///     #[repr(u8)]
///     enum RepeatedEnum1 {
///         #(Variant~N,)*
///     }
/// });
///
/// seq!(N in 0..=1 {
///     mod repeated_mod_~N {
///         #(
///             #[repr(u8)]
///             pub enum RepeatedEnum~N {
///                 #(Variant~N,)*
///             }
///         )*
///     }
/// });
///
/// let _ = RepeatedEnum0::Variant0;
/// let _ = RepeatedEnum0::Variant1;
///
/// let _ = RepeatedEnum1::Variant0;
/// let _ = RepeatedEnum1::Variant1;
///
/// let _ = repeated_mod_0::RepeatedEnum0::Variant0;
/// let _ = repeated_mod_0::RepeatedEnum0::Variant1;
///
/// let _ = repeated_mod_0::RepeatedEnum1::Variant0;
/// let _ = repeated_mod_0::RepeatedEnum1::Variant1;
///
/// let _ = repeated_mod_1::RepeatedEnum0::Variant0;
/// let _ = repeated_mod_1::RepeatedEnum0::Variant1;
///
/// let _ = repeated_mod_1::RepeatedEnum1::Variant0;
/// let _ = repeated_mod_1::RepeatedEnum1::Variant1;
/// ```
#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let data = parse_macro_input!(input as SeqData);
    let token_buffer = TokenBuffer::new2(data.body_token_stream);

    let parsed_token_streams =
        parse::token_stream_at_cursor(token_buffer.begin(), data.ident.as_ref());

    construct::output_token_stream(&parsed_token_streams, &data.range_kind).into()
}
