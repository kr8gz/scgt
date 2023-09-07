use std::collections::BTreeSet;

use chumsky::prelude::*;

use crate::helper_functions::*;

type Err<'a> = Rich<'a, char>;
type Extra<'a> = extra::Full<Err<'a>, State<'a>, ()>;

struct State<'a> {
    helper_functions: BTreeSet<HelperFunction>,
    variables: BTreeSet<String>,
    source: &'a str,
    indent_level: usize,
}

impl<'a> State<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            helper_functions: BTreeSet::new(),
            variables: BTreeSet::new(),
            source,
            indent_level: 0,
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

struct Value(String, PrintBehavior);

pub fn parse(code: &str) -> ParseResult<String, Err<'_>> {
    let mut state = State::new(code);
    parser().parse_with_state(code, &mut state)
}

fn parser<'a>() -> impl Parser<'a, &'a str, String, Extra<'a>> + Clone {
    choice((
        just("SCGT").map(|_| {
            let _ = open::that("https://github.com/kr8gz/scgt/");
            ":)".to_string()
        }),

        recursive(|block| {
            let ident = one_of("abcdefghijklmnopqrstuvwxyz")
                .repeated().at_least(1)
                .collect::<String>()
                .or(just('_').ignore_then(text::ident()).map(String::from))
                .labelled("identifier");

            let expression = recursive(|expression| {
                let int = text::int(10).map(String::from);

                let float = text::int(10).slice().or_not()
                    .then_ignore(just('.'))
                    .then(text::digits(10).slice().or_not())
                    .filter(|(bef, aft)| bef.as_ref().or(aft.as_ref()).is_some())
                    .map(|(bef, aft)| format!("{}.{}", bef.unwrap_or("0"), aft.unwrap_or("0")));

                let char_literal = just('\'')
                    .ignore_then(any())
                    .map(|v| format!("\"{v}\""));

                let string_char = choice((
                    just('\\')
                        .ignore_then(one_of("nrt`\\").or_not())
                        .map(|c| format!("\\{}", c.unwrap_or('\\'))),

                    any()
                        .and_is(just('`').ignored().or(text::newline()).not())
                        .map(String::from)
                ));

                let string = choice((
                    // `...`
                    string_char
                        .repeated()
                        .collect::<Vec<_>>()
                        .delimited_by(just('`'), just('`').or_not()),
                    // \...` (allows newlines)
                    string_char.or(text::newline().to("\n".to_string()))
                        .repeated()
                        .collect::<Vec<_>>()
                        .delimited_by(just('\\'), just('`').ignored().or(end()))
                ))
                .map(|v| format!("\"{}\"", v.into_iter().collect::<String>()));

                let value_ident = ident.map_with_state(|v, _, state: &mut State| {
                    let helper = state.add_helper(HelperFunction::Get);
                    let code = format!("{helper}({v})");

                    state.variables.insert(v);

                    code
                });

                let type_indicator = just('@')
                    .ignore_then(ident)
                    .map(|name| format!("@{name}"));

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
                    hardcoded,
                ))
                .map(|v| Value(v, PrintBehavior::Implicit));

                let invert = just('!')
                    .ignore_then(expression.clone())
                    .map_with_state(|Value(v, _), _, state: &mut State| {
                        let helper = state.add_helper(HelperFunction::Invert);
                        format!("{helper}({v})")
                    });

                let explicit_print = just('$')
                    .ignore_then(expression.clone())
                    .map_with_state(|Value(v, _), _, state: &mut State| {
                        let helper = state.add_helper(HelperFunction::ExplicitPrint);
                        format!("{helper}({v})")
                    });

                let explicit_print_values = choice((
                    invert,
                    explicit_print,
                ))
                .map(|v| Value(v, PrintBehavior::Explicit));

                let value = implicit_print_values.or(explicit_print_values).labelled("value");

                // TODO add space " " handling

                // TODO fold binary operators
                // if a fold occurs then printbehavior -> implicit
                value
            })
            .labelled("expression");

            let statement = choice((
                expression.map(|Value(v, print)| match print {
                    PrintBehavior::Implicit => format!("$.print({v})"),
                    PrintBehavior::Explicit => v,
                }),
            ))
            .map_with_state(|s, span: SimpleSpan, state: &mut State| {
                let indent = " ".repeat(4 * state.indent_level);
                format!("{indent}// {}\n{indent}{s}", &state.source[span.start..span.end])
            });

            statement
                .padded_by(just('\n').or_not())
                .repeated()
                .collect::<Vec<_>>()
                .map(|v| v.join("\n\n"))
        })
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
