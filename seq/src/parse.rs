use proc_macro2::{
    extra::DelimSpan, Delimiter, Group, Literal, Punct, Spacing, Span, TokenStream, TokenTree,
};
use std::ops::ControlFlow;
use syn::{buffer::Cursor, Ident, Lifetime, Lit};

pub enum ParsedTokenStream {
    RepeatedSection(Vec<ParsedTokenStream>),
    NonReplaceableTokenStream(ParsedNonReplaceableTokenStream),
    ReplaceableStandaloneIdent(ParsedReplaceableStandaloneIdent),
    ReplaceableComplexIdent(ParsedReplaceableComplexIdent),
    ReplaceableGroup(ParsedReplaceableGroup),
}

pub struct ParsedNonReplaceableTokenStream {
    pub stream: TokenStream,
}

pub struct ParsedReplaceableStandaloneIdent {
    pub span: Span,
}

pub struct ParsedReplaceableComplexIdent {
    pub components: Vec<ParsedReplaceableComplexIdentComponent>,
    pub span: Span,
}

pub enum ParsedReplaceableComplexIdentComponent {
    FixedComponent(String),
    ReplaceableComponent,
}

pub struct ParsedReplaceableGroup {
    pub delim: proc_macro2::Delimiter,
    pub delim_span: DelimSpan,
    pub parsed_token_streams: Vec<ParsedTokenStream>,
}

pub fn token_stream_at_cursor(
    cursor: Cursor<'_>,
    ident_to_replace: Option<&Ident>,
) -> Vec<ParsedTokenStream> {
    let mut parse_data = ParseData {
        ident_to_replace,
        encountered_top_level_ident_to_replace: false,
        encountered_repeated_section: false,
        parsed_token_streams: Vec::new(),
        stack: vec![Frame {
            cursor,
            parsed_token_streams: Vec::new(),
            is_in_repeated_section: false,
            additional_data: None,
        }],
    };

    while let Some(mut frame) = parse_data.stack.pop() {
        loop {
            if let Some((ident, next_cursor)) = frame.cursor.ident() {
                parse_ident(ident, &mut parse_data, &mut frame, next_cursor);
            } else if let Some((punct, next_cursor)) = frame.cursor.punct() {
                let ControlFlow::Continue(f) =
                    parse_repeated_section(&punct, &mut parse_data, frame, next_cursor)
                else {
                    break;
                };

                frame = f;
                parse_punct(punct, &mut frame, next_cursor);
            } else if let Some((literal, next_cursor)) = frame.cursor.literal() {
                parse_literal(literal, &mut frame, next_cursor);
            } else if let Some((lifetime, next_cursor)) = frame.cursor.lifetime() {
                parse_lifetime(lifetime, &mut frame, next_cursor);
            } else if let Some((group_cursor, delim, delim_span, next_cursor)) =
                frame.cursor.any_group()
            {
                parse_group(
                    group_cursor,
                    delim,
                    delim_span,
                    &mut parse_data,
                    frame,
                    next_cursor,
                );

                break;
            } else {
                finish_frame(&mut parse_data, frame);
                break;
            }
        }
    }

    parse_data.parsed_token_streams
}

struct Frame<'a> {
    cursor: Cursor<'a>,
    parsed_token_streams: Vec<ParsedTokenStream>,
    is_in_repeated_section: bool,
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

struct ParseData<'a> {
    ident_to_replace: Option<&'a Ident>,
    encountered_top_level_ident_to_replace: bool,
    encountered_repeated_section: bool,
    parsed_token_streams: Vec<ParsedTokenStream>,
    stack: Vec<Frame<'a>>,
}

fn parse_ident<'a>(
    ident: Ident,
    parse_data: &mut ParseData<'_>,
    frame: &mut Frame<'a>,
    next_cursor: Cursor<'a>,
) {
    if let Some(ident_to_replace) = parse_data.ident_to_replace {
        if &ident == ident_to_replace {
            frame
                .parsed_token_streams
                .push(ParsedTokenStream::ReplaceableStandaloneIdent(
                    ParsedReplaceableStandaloneIdent { span: ident.span() },
                ));

            if !frame.is_in_repeated_section {
                parse_data.encountered_top_level_ident_to_replace = true;
            }

            frame.cursor = next_cursor;
            return;
        } else if let Some((replaceable_complex_ident, next_cursor)) =
            parse_replaceable_complex_ident(next_cursor, &ident, ident_to_replace)
        {
            frame
                .parsed_token_streams
                .push(ParsedTokenStream::ReplaceableComplexIdent(
                    replaceable_complex_ident,
                ));

            if !frame.is_in_repeated_section {
                parse_data.encountered_top_level_ident_to_replace = true;
            }

            frame.cursor = next_cursor;
            return;
        }
    }

    let stream = TokenStream::from(TokenTree::from(ident));

    append_to_last_non_replaceable_token_stream_or_push_new(
        &mut frame.parsed_token_streams,
        stream,
    );

    frame.cursor = next_cursor;
}

fn parse_repeated_section<'a>(
    punct: &Punct,
    parse_data: &mut ParseData<'a>,
    frame: Frame<'a>,
    next_cursor: Cursor<'a>,
) -> ControlFlow<(), Frame<'a>> {
    if punct.as_char() != '#' {
        return ControlFlow::Continue(frame);
    }

    let Some((group_cursor, _, next_cursor)) = next_cursor.group(Delimiter::Parenthesis) else {
        return ControlFlow::Continue(frame);
    };

    let Some((punct, next_cursor)) = next_cursor.punct() else {
        return ControlFlow::Continue(frame);
    };

    if punct.as_char() != '*' {
        return ControlFlow::Continue(frame);
    }

    parse_data.stack.push(Frame {
        cursor: next_cursor,
        parsed_token_streams: frame.parsed_token_streams,
        is_in_repeated_section: frame.is_in_repeated_section,
        additional_data: frame.additional_data,
    });

    parse_data.stack.push(Frame {
        cursor: group_cursor,
        parsed_token_streams: Vec::new(),
        is_in_repeated_section: true,
        additional_data: Some(AdditionalFrameData::RepeatedSection),
    });

    parse_data.encountered_repeated_section = true;

    ControlFlow::Break(())
}

fn parse_punct<'a>(punct: Punct, frame: &mut Frame<'a>, next_cursor: Cursor<'a>) {
    let stream = TokenStream::from(TokenTree::from(punct));

    append_to_last_non_replaceable_token_stream_or_push_new(
        &mut frame.parsed_token_streams,
        stream,
    );

    frame.cursor = next_cursor;
}

fn parse_literal<'a>(literal: Literal, frame: &mut Frame<'a>, next_cursor: Cursor<'a>) {
    let stream = TokenStream::from(TokenTree::from(literal));

    append_to_last_non_replaceable_token_stream_or_push_new(
        &mut frame.parsed_token_streams,
        stream,
    );

    frame.cursor = next_cursor;
}

fn parse_lifetime<'a>(lifetime: Lifetime, frame: &mut Frame<'a>, next_cursor: Cursor<'a>) {
    let mut punct = Punct::new('\'', Spacing::Joint);
    punct.set_span(lifetime.apostrophe);

    let mut stream = TokenStream::from(TokenTree::from(punct));
    stream.extend(TokenStream::from(TokenTree::from(lifetime.ident)));

    append_to_last_non_replaceable_token_stream_or_push_new(
        &mut frame.parsed_token_streams,
        stream,
    );

    frame.cursor = next_cursor;
}

fn parse_group<'a>(
    group_cursor: Cursor<'a>,
    delim: Delimiter,
    delim_span: DelimSpan,
    parse_data: &mut ParseData<'a>,
    frame: Frame<'_>,
    next_cursor: Cursor<'a>,
) {
    parse_data.stack.push(Frame {
        cursor: next_cursor,
        parsed_token_streams: frame.parsed_token_streams,
        is_in_repeated_section: frame.is_in_repeated_section,
        additional_data: frame.additional_data,
    });

    let group_frame_data = GroupFrameData { delim, delim_span };

    parse_data.stack.push(Frame {
        cursor: group_cursor,
        parsed_token_streams: Vec::new(),
        is_in_repeated_section: frame.is_in_repeated_section,
        additional_data: Some(AdditionalFrameData::Group(group_frame_data)),
    });
}

fn finish_frame(parse_data: &mut ParseData<'_>, frame: Frame<'_>) {
    if let Some(previous_frame) = parse_data.stack.last_mut() {
        finish_top_frame(frame, previous_frame);
    } else {
        finish_bottom_frame(parse_data, frame);
    }
}

fn finish_top_frame(frame: Frame<'_>, previous_frame: &mut Frame<'_>) {
    match frame.additional_data {
        Some(AdditionalFrameData::Group(group_data)) => {
            finish_group_top_frame(&group_data, frame.parsed_token_streams, previous_frame);
        }
        Some(AdditionalFrameData::RepeatedSection) => {
            finish_repeated_section_top_frame(frame.parsed_token_streams, previous_frame);
        }
        None => {
            finish_regular_top_frame(frame.parsed_token_streams, previous_frame);
        }
    }
}

fn finish_group_top_frame(
    group_data: &GroupFrameData,
    mut parsed_token_streams: Vec<ParsedTokenStream>,
    previous_frame: &mut Frame<'_>,
) {
    if parsed_token_streams.len() == 1 {
        if let Some(parsed_token_stream) = parsed_token_streams.pop() {
            if let ParsedTokenStream::NonReplaceableTokenStream(parsed_stream) = parsed_token_stream
            {
                let mut group = Group::new(group_data.delim, parsed_stream.stream);

                group.set_span(group_data.delim_span.join());

                let stream = TokenStream::from(TokenTree::from(group));

                append_to_last_non_replaceable_token_stream_or_push_new(
                    &mut previous_frame.parsed_token_streams,
                    stream,
                );
            } else {
                parsed_token_streams.push(parsed_token_stream);

                previous_frame
                    .parsed_token_streams
                    .push(ParsedTokenStream::ReplaceableGroup(
                        ParsedReplaceableGroup {
                            delim: group_data.delim,
                            delim_span: group_data.delim_span,
                            parsed_token_streams,
                        },
                    ));
            }
        }
    } else if parsed_token_streams.is_empty() {
        let mut group = Group::new(group_data.delim, TokenStream::new());
        group.set_span(group_data.delim_span.join());

        let stream = TokenStream::from(TokenTree::from(group));

        append_to_last_non_replaceable_token_stream_or_push_new(
            &mut previous_frame.parsed_token_streams,
            stream,
        );
    } else {
        previous_frame
            .parsed_token_streams
            .push(ParsedTokenStream::ReplaceableGroup(
                ParsedReplaceableGroup {
                    delim: group_data.delim,
                    delim_span: group_data.delim_span,
                    parsed_token_streams,
                },
            ));
    }
}

fn finish_repeated_section_top_frame(
    parsed_token_streams: Vec<ParsedTokenStream>,
    previous_frame: &mut Frame<'_>,
) {
    previous_frame
        .parsed_token_streams
        .push(ParsedTokenStream::RepeatedSection(parsed_token_streams));
}

fn finish_regular_top_frame(
    parsed_token_streams: Vec<ParsedTokenStream>,
    previous_frame: &mut Frame<'_>,
) {
    previous_frame
        .parsed_token_streams
        .extend(parsed_token_streams);
}

fn finish_bottom_frame(parse_data: &mut ParseData<'_>, frame: Frame<'_>) {
    if parse_data.encountered_repeated_section && !parse_data.encountered_top_level_ident_to_replace
    {
        parse_data.parsed_token_streams = frame.parsed_token_streams;
    } else {
        parse_data
            .parsed_token_streams
            .push(ParsedTokenStream::RepeatedSection(
                frame.parsed_token_streams,
            ));
    }
}

fn append_to_last_non_replaceable_token_stream_or_push_new(
    parsed_token_streams: &mut Vec<ParsedTokenStream>,
    stream: TokenStream,
) {
    if let Some(ParsedTokenStream::NonReplaceableTokenStream(parsed_stream)) =
        parsed_token_streams.last_mut()
    {
        parsed_stream.stream.extend(stream);
    } else {
        parsed_token_streams.push(ParsedTokenStream::NonReplaceableTokenStream(
            ParsedNonReplaceableTokenStream { stream },
        ));
    }
}

fn parse_replaceable_complex_ident<'a>(
    cursor: Cursor<'a>,
    current_ident: &Ident,
    ident_to_replace: &Ident,
) -> Option<(ParsedReplaceableComplexIdent, Cursor<'a>)> {
    let (punct, next_cursor) = cursor.punct()?;

    if punct.as_char() != '~' {
        return None;
    }

    let (ident, next_cursor) = next_cursor.ident()?;

    if &ident != ident_to_replace {
        return None;
    }

    let mut parse_data = ParseComplexIdentData {
        components: vec![
            ParsedReplaceableComplexIdentComponent::FixedComponent(current_ident.to_string()),
            ParsedReplaceableComplexIdentComponent::ReplaceableComponent,
        ],
        span: current_ident
            .span()
            .join(punct.span())
            .and_then(|s| s.join(ident.span())),
        current_cursor: next_cursor,
        needs_ident_to_replace: false,
    };

    while let Some((punct, next_cursor)) = parse_data.current_cursor.punct() {
        if punct.as_char() != '~' {
            break;
        }

        if let Some((ident, next_cursor)) = next_cursor.ident() {
            if parse_replaceable_complex_ident_subident(
                &ident,
                ident_to_replace,
                &mut parse_data,
                next_cursor,
                &punct,
            )
            .is_break()
            {
                break;
            }
        } else if let Some((another_punct, next_cursor)) = next_cursor.punct() {
            if parse_replaceable_complex_ident_punct(
                &another_punct,
                ident_to_replace,
                &mut parse_data,
                next_cursor,
                &punct,
            )
            .is_break()
            {
                break;
            }
        } else if let Some((literal, next_cursor)) = next_cursor.literal() {
            if parse_replaceable_complex_ident_literal(
                literal,
                &mut parse_data,
                next_cursor,
                &punct,
            )
            .is_break()
            {
                break;
            }
        } else {
            break;
        }
    }

    Some((
        ParsedReplaceableComplexIdent {
            components: parse_data.components,
            span: parse_data.span.unwrap_or(current_ident.span()),
        },
        parse_data.current_cursor,
    ))
}

struct ParseComplexIdentData<'a> {
    components: Vec<ParsedReplaceableComplexIdentComponent>,
    span: Option<Span>,
    current_cursor: Cursor<'a>,
    needs_ident_to_replace: bool,
}

fn parse_replaceable_complex_ident_subident<'a>(
    ident: &Ident,
    ident_to_replace: &Ident,
    parse_data: &mut ParseComplexIdentData<'a>,
    next_cursor: Cursor<'a>,
    first_punct: &Punct,
) -> ControlFlow<()> {
    if ident == ident_to_replace {
        if !parse_data.needs_ident_to_replace {
            return ControlFlow::Break(());
        }

        parse_data
            .components
            .push(ParsedReplaceableComplexIdentComponent::ReplaceableComponent);

        parse_data.needs_ident_to_replace = false;
    } else {
        if parse_data.needs_ident_to_replace {
            return ControlFlow::Break(());
        }

        parse_data
            .components
            .push(ParsedReplaceableComplexIdentComponent::FixedComponent(
                ident.to_string(),
            ));

        parse_data.needs_ident_to_replace = true;
    }

    parse_data.span = parse_data
        .span
        .and_then(|s| s.join(first_punct.span()))
        .and_then(|s| s.join(ident.span()));

    parse_data.current_cursor = next_cursor;
    ControlFlow::Continue(())
}

fn parse_replaceable_complex_ident_punct<'a>(
    punct: &Punct,
    ident_to_replace: &Ident,
    parse_data: &mut ParseComplexIdentData<'a>,
    next_cursor: Cursor<'a>,
    first_punct: &Punct,
) -> ControlFlow<()> {
    if punct.as_char() != '~' {
        return ControlFlow::Break(());
    }

    let Some((another_ident, next_cursor)) = next_cursor.ident() else {
        return ControlFlow::Break(());
    };

    if &another_ident != ident_to_replace {
        return ControlFlow::Break(());
    }

    parse_data
        .components
        .push(ParsedReplaceableComplexIdentComponent::ReplaceableComponent);

    parse_data.span = parse_data
        .span
        .and_then(|s| s.join(first_punct.span()))
        .and_then(|s| s.join(punct.span()))
        .and_then(|s| s.join(another_ident.span()));

    parse_data.current_cursor = next_cursor;
    parse_data.needs_ident_to_replace = false;

    ControlFlow::Continue(())
}

fn parse_replaceable_complex_ident_literal<'a>(
    literal: Literal,
    parse_data: &mut ParseComplexIdentData<'a>,
    next_cursor: Cursor<'a>,
    first_punct: &Punct,
) -> ControlFlow<()> {
    if parse_data.needs_ident_to_replace {
        return ControlFlow::Break(());
    }

    let literal_as_string = literal.to_string();
    let literal_span = literal.span();
    let token_stream = TokenStream::from(TokenTree::from(literal));

    let Ok(parsed_lit) = syn::parse2::<Lit>(token_stream) else {
        return ControlFlow::Break(());
    };

    match parsed_lit {
        Lit::Int(_) => (),
        Lit::Float(_) => {
            if literal_as_string.contains('.') {
                return ControlFlow::Break(());
            }
        }
        _ => return ControlFlow::Break(()),
    }

    parse_data
        .components
        .push(ParsedReplaceableComplexIdentComponent::FixedComponent(
            literal_as_string,
        ));

    parse_data.span = parse_data
        .span
        .and_then(|s| s.join(first_punct.span()))
        .and_then(|s| s.join(literal_span));

    parse_data.current_cursor = next_cursor;
    parse_data.needs_ident_to_replace = true;

    ControlFlow::Continue(())
}
