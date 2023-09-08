use std::collections::BTreeSet;

use chumsky::prelude::*;

use crate::helper_functions::*;

type Err<'a> = Rich<'a, char>;
type Extra<'a> = extra::Full<Err<'a>, State<'a>, ()>;

macro_rules! parser_type {
    ( $ret_t:ty ) => {
        impl Parser<'a, &'a str, $ret_t, Extra<'a>> + Clone
    }
}

struct State<'a> {
    helper_functions: BTreeSet<HelperFunction>,
    variables: BTreeSet<String>,
    source: &'a str,
}

impl<'a> State<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            helper_functions: BTreeSet::new(),
            variables: BTreeSet::new(),
            source,
        }
    }

    fn add_helper(&mut self, helper: HelperFunction) -> &'static str {
        self.helper_functions.insert(helper);
        helper.spwn_name()
    }
}

enum PrintBehavior {
    Implicit,
    Explicit,
}

struct SpwnCode {
    code: String,
    span: SimpleSpan,
    print: PrintBehavior,
}

impl SpwnCode {
    fn implicit_print(code: String, span: SimpleSpan) -> Self {
        Self { code, span, print: PrintBehavior::Implicit }
    }

    fn explicit_print(code: String, span: SimpleSpan) -> Self {
        Self { code, span, print: PrintBehavior::Explicit }
    }
}

pub fn parse(code: &str) -> ParseResult<String, Err<'_>> {
    let mut state = State::new(code);
    parser().parse_with_state(code, &mut state)
}

fn parser<'a>() -> parser_type!(String) {
    let global = recursive(|block| {
        let ident = one_of("abcdefghijklmnopqrstuvwxyz")
            .repeated().at_least(1)
            .collect::<String>()
            .or(just('_').ignore_then(text::ident()).map(String::from))
            .labelled("identifier");

        let closing = choice((
            just(';').ignored(),
            text::newline().rewind(),
            end()
        ));

        let expression = recursive(|expression| {
            let int = text::int(10).map(String::from);

            let float = text::int(10).slice().or_not()
                .then_ignore(just('.'))
                .then(text::digits(10).slice().or_not())
                .filter(|(bef, aft)| bef.as_ref().or(aft.as_ref()).is_some())
                .map(|(bef, aft)| format!("{}.{}", bef.unwrap_or("0"), aft.unwrap_or("0")));

            let char_literal = just('\'')
                .ignore_then(any())
                .map(|c| format!("\"{c}\""));

            let string_char = choice((
                just('\\')
                    .ignore_then(
                        select! {
                            'n' => "\\n",
                            'r' => "\\r",
                            't' => "\\t",
                            '`' => "`",
                            '\\' => "\\\\",
                        }
                        .or_not()
                    )
                    .map(|c| c.unwrap_or("\\").to_string()),

                any()
                    .and_is(just('`').ignored().or(text::newline()).not())
                    .map(String::from)
            ));

            let string = choice((
                // `...`
                string_char
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(
                        just('`'),
                        choice((
                            just('`').ignored(),
                            text::newline().rewind(),
                            end()
                        ))
                    ),
                // \...` (allows newlines)
                string_char.or(text::newline().to("\n".to_string()))
                    .repeated()
                    .collect::<Vec<_>>()
                    .delimited_by(just('\\'), just('`').ignored().or(end()))
            ))
            .map(|v| format!("\"{}\"", v.into_iter().collect::<String>()));

            let value_ident = ident.map_with_state(|s, _, state: &mut State| {
                let helper = state.add_helper(HelperFunction::Get);
                let code = format!("{helper}({s})");

                state.variables.insert(s);

                code
            });

            let type_indicator = just('@')
                .ignore_then(ident)
                .map(|name| format!("@{name}"));

            let inner_block = finish_block(block.clone(), true, ",\n", true)
                .delimited_by(just('('), closing)
                .map_with_state(|s, _, state: &mut State| {
                    let helper = state.add_helper(HelperFunction::Last);
                    format!("{helper}([{s}])")
                });

            let invert = just('!')
                .ignore_then(expression.clone())
                .map_with_state(|SpwnCode { code, .. }, _, state: &mut State| {
                    let helper = state.add_helper(HelperFunction::Invert);
                    format!("{helper}({code})")
                });

            let hardcoded = select! {
                'A' => "[]",
                'B' => "?b",
                'C' => "?c",
                'D' => "?i",
                'F' => "false",
                'G' => "?g",
                'N' => "null",
                'S' => "\"\"",
                'T' => "true",
            }
            .map(String::from);

            let implicit_print_values = choice((
                int, float,
                char_literal, string,
                value_ident, type_indicator,
                inner_block,
                invert,
                hardcoded,
            ))
            .map_with_span(SpwnCode::implicit_print);

            let explicit_print = just('$')
                .ignore_then(expression.clone())
                .map_with_state(|SpwnCode { code, .. }, _, state: &mut State| {
                    let helper = state.add_helper(HelperFunction::Print);
                    format!("{helper}({code})")
                });

            // TODO this return_last should be true if it is still accessible from statement level
            let infinite_loop = finish_block(block.clone(), false, "\n", true)
                .delimited_by(just('L'), closing)
                .map(|s| format!("while true {{{s}}}"));

            let explicit_print_values = choice((
                explicit_print,
                infinite_loop,
            ))
            .map_with_span(SpwnCode::explicit_print);

            let value = implicit_print_values.or(explicit_print_values).labelled("value");

            // TODO add space " " handling

            // TODO fold binary operators
            // if a fold occurs then printbehavior -> implicit
            value
        })
        .labelled("expression");

        choice((
            expression,
        ))
        .padded_by(just('\n').or_not())
        .repeated()
        .collect::<Vec<_>>()
        .labelled("statement")
    });

    choice((
        just("SCGT").map(|_| {
            let _ = open::that("https://github.com/kr8gz/scgt/");
            ":)".to_string()
        }),

        finish_block(global, false, "\n\n", false)
            .map_with_state(|mut code, _, state: &mut State| {
                if !state.variables.is_empty() {
                    code = format!("{}\n{}\n{code}",
                        "// Initialize variables used",
                        state.variables
                            .iter()
                            .rfold(String::new(), |rest, var| {
                                format!("let {var} = null\n{rest}")
                            })
                    );
                }

                if !state.helper_functions.is_empty() {
                    code = format!("{}\n{}{code}",
                        "// Automatically generated SCGT helper functions",
                        state.helper_functions
                            .iter()
                            .rfold(String::new(), |rest, helper| {
                                format!("{} = {}\n\n{rest}", helper.spwn_name(), helper.spwn_impl())
                            })
                    );
                }

                code
            })
    ))
}

fn finish_block<'a>(
    statements: parser_type!(Vec<SpwnCode>),
    return_last: bool,
    delimiter: &'a str,
    format: bool,
) -> parser_type!(String) {
    statements.map_with_state(move |v, _, state: &mut State| {
        if v.is_empty() {
            String::new()
        } else {
            let last_index = v.len() - 1;
            let indent = if format { "    " } else { "" };
    
            let code = v.into_iter().enumerate().map(|(i, SpwnCode { mut code, span, print })| {
                let comment = state.source[span.start..span.end]
                    .lines()
                    .map(|line| format!("{indent}// {line}\n"))
                    .collect::<String>();
    
                if return_last && i == last_index {
                    // TODO check here again once returning values is a thing
                } else {
                    code = match print {
                        PrintBehavior::Explicit => code,
                        PrintBehavior::Implicit => {
                            let helper = state.add_helper(HelperFunction::Print);
                            format!("{helper}({code})")
                        }
                    };
                }

                code = code
                    .lines()
                    .map(|line| format!("{indent}{line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
            
                format!("{comment}{code}")
            })
            .collect::<Vec<_>>()
            .join(delimiter);

            if format {
                format!("\n{code}\n")
            } else {
                code
            }
        }
    })
}
