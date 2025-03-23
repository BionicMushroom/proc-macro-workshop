use crate::parse::{
    ParsedNonReplaceableTokenStream, ParsedReplaceableComplexIdent,
    ParsedReplaceableComplexIdentComponent, ParsedReplaceableGroup,
    ParsedReplaceableStandaloneIdent, ParsedTokenStream,
};
use crate::seq_data::RangeKind;
use proc_macro2::{extra::DelimSpan, Delimiter, Group, Literal, TokenStream, TokenTree};
use syn::Ident;

pub fn output_token_stream(
    parsed_token_streams: &[ParsedTokenStream],
    range_kind: &RangeKind,
) -> TokenStream {
    let mut construct_data = ConstructData {
        output_token_stream: TokenStream::new(),
        stack: vec![Frame {
            output_token_stream: TokenStream::new(),
            parsed_token_streams,
            repetition_index: 0,
            additional_data: None,
        }],
    };

    'stack_loop: while let Some(mut frame) = construct_data.stack.pop() {
        for (parsed_stream_index, parsed_stream) in frame.parsed_token_streams.iter().enumerate() {
            match parsed_stream {
                ParsedTokenStream::RepeatedSection(section) => {
                    construct_repeated_section(
                        section,
                        range_kind,
                        &mut construct_data,
                        parsed_stream_index,
                        frame,
                    );

                    continue 'stack_loop;
                }
                ParsedTokenStream::NonReplaceableTokenStream(parsed_stream) => {
                    construct_non_replaceable_token_stream(parsed_stream, &mut frame);
                }
                ParsedTokenStream::ReplaceableStandaloneIdent(parsed_ident) => {
                    construct_replaceable_standalone_ident(parsed_ident, &mut frame);
                }
                ParsedTokenStream::ReplaceableComplexIdent(parsed_ident) => {
                    construct_replaceable_complex_ident(parsed_ident, &mut frame);
                }
                ParsedTokenStream::ReplaceableGroup(parsed_group) => {
                    construct_replaceable_group(
                        parsed_group,
                        &mut construct_data,
                        parsed_stream_index,
                        frame,
                    );

                    continue 'stack_loop;
                }
            }
        }

        finish_frame(&mut construct_data, frame);
    }

    construct_data.output_token_stream
}

struct Frame<'a> {
    output_token_stream: TokenStream,
    parsed_token_streams: &'a [ParsedTokenStream],
    repetition_index: u128,
    additional_data: Option<AdditionalFrameData>,
}

enum AdditionalFrameData {
    Group(GroupFrameData),
    RepeatedSection,
}

struct GroupFrameData {
    delim: Delimiter,
    delim_span: DelimSpan,
}

struct ConstructData<'a> {
    output_token_stream: TokenStream,
    stack: Vec<Frame<'a>>,
}

fn construct_repeated_section<'a>(
    repeated_section: &'a Vec<ParsedTokenStream>,
    range_kind: &RangeKind,
    construct_data: &mut ConstructData<'a>,
    parsed_stream_index: usize,
    frame: Frame<'a>,
) {
    let parsed_token_streams = &frame.parsed_token_streams[parsed_stream_index + 1..];

    construct_data.stack.push(Frame {
        output_token_stream: frame.output_token_stream,
        parsed_token_streams,
        repetition_index: frame.repetition_index,
        additional_data: frame.additional_data,
    });

    repeat_in_reverse(range_kind, |i| {
        construct_data.stack.push(Frame {
            output_token_stream: TokenStream::new(),
            parsed_token_streams: repeated_section,
            repetition_index: i,
            additional_data: Some(AdditionalFrameData::RepeatedSection),
        });
    });
}

fn construct_non_replaceable_token_stream(
    parsed_stream: &ParsedNonReplaceableTokenStream,
    frame: &mut Frame<'_>,
) {
    frame
        .output_token_stream
        .extend(parsed_stream.stream.clone());
}

fn construct_replaceable_standalone_ident(
    parsed_ident: &ParsedReplaceableStandaloneIdent,
    frame: &mut Frame<'_>,
) {
    let mut literal = Literal::u128_unsuffixed(frame.repetition_index);
    literal.set_span(parsed_ident.span);

    frame
        .output_token_stream
        .extend(TokenStream::from(TokenTree::from(literal)));
}

fn construct_replaceable_complex_ident(
    parsed_ident: &ParsedReplaceableComplexIdent,
    frame: &mut Frame<'_>,
) {
    let repetition_index_as_string = frame.repetition_index.to_string();

    let full_ident_name =
        parsed_ident
            .components
            .iter()
            .fold(String::new(), |mut ident_name, component| match component {
                ParsedReplaceableComplexIdentComponent::FixedComponent(c) => {
                    ident_name.push_str(c);
                    ident_name
                }
                ParsedReplaceableComplexIdentComponent::ReplaceableComponent => {
                    ident_name.push_str(&repetition_index_as_string);
                    ident_name
                }
            });

    let ident = Ident::new(&full_ident_name, parsed_ident.span);

    frame
        .output_token_stream
        .extend(TokenStream::from(TokenTree::from(ident)));
}

fn construct_replaceable_group<'a>(
    parsed_group: &'a ParsedReplaceableGroup,
    construct_data: &mut ConstructData<'a>,
    parsed_stream_index: usize,
    frame: Frame<'a>,
) {
    let parsed_token_streams = &frame.parsed_token_streams[parsed_stream_index + 1..];

    construct_data.stack.push(Frame {
        output_token_stream: frame.output_token_stream,
        parsed_token_streams,
        repetition_index: frame.repetition_index,
        additional_data: frame.additional_data,
    });

    let group_data = GroupFrameData {
        delim: parsed_group.delim,
        delim_span: parsed_group.delim_span,
    };

    construct_data.stack.push(Frame {
        output_token_stream: TokenStream::new(),
        parsed_token_streams: &parsed_group.parsed_token_streams,
        repetition_index: frame.repetition_index,
        additional_data: Some(AdditionalFrameData::Group(group_data)),
    });
}

fn finish_frame(construct_data: &mut ConstructData<'_>, mut frame: Frame<'_>) {
    if let Some(previous_frame) = construct_data.stack.last_mut() {
        match frame.additional_data.take() {
            Some(AdditionalFrameData::Group(group_data)) => {
                let mut group = Group::new(group_data.delim, frame.output_token_stream);
                group.set_span(group_data.delim_span.join());

                previous_frame
                    .output_token_stream
                    .extend(TokenStream::from(TokenTree::from(group)));
            }
            Some(AdditionalFrameData::RepeatedSection) | None => {
                previous_frame
                    .output_token_stream
                    .extend(frame.output_token_stream);
            }
        }
    } else {
        construct_data.output_token_stream = frame.output_token_stream;
    }
}

fn repeat_in_reverse(range_kind: &RangeKind, mut f: impl FnMut(u128)) {
    match range_kind {
        RangeKind::Exclusive(r) => {
            for i in r.clone().rev() {
                f(i);
            }
        }
        RangeKind::Inclusive(r) => {
            for i in r.clone().rev() {
                f(i);
            }
        }
    }
}
