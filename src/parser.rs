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
    recursive(|block| {
        let ident = one_of("abcdefghijklmnopqrstuvwxyz").repeated().at_least(1)
            .collect::<String>()
            .or(just("_").ignore_then(text::ident()).map(str::to_string))
            .labelled("identifier");

        let expression = recursive(|expression| {
            let value_ident = ident.map_with_state(|v, _, state: &mut State| {
                let helper = state.add_helper(HelperFunction::Get);
                let code = format!("{helper}({v})");

                state.variables.insert(v);

                code
            });

            let type_indicator = just("@")
                .ignore_then(ident)
                .map(|name| format!("@{name}"));

            let int = text::int(10).map(str::to_string);

            let float = text::int(10).slice().or_not()
                .then_ignore(just("."))
                .then(text::digits(10).slice().or_not())
                .filter(|(bef, aft)| bef.as_ref().or(aft.as_ref()).is_some())
                .map(|(bef, aft)| format!("{}.{}", bef.unwrap_or("0"), aft.unwrap_or("0")));

            let invert = just("!")
                .ignore_then(expression.clone())
                .map_with_state(|Value(v, _), _, state: &mut State| {
                    let helper = state.add_helper(HelperFunction::Invert);
                    format!("{helper}({v})")
                });

            let explicit_print = just("$")
                .ignore_then(expression.clone())
                .map_with_state(|Value(v, _), _, state: &mut State| {
                    let helper = state.add_helper(HelperFunction::ExplicitPrint);
                    format!("{helper}({v})")
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
            .map(str::to_string);

            // TODO add space " " handling

            let implicit_print_values = choice((
                value_ident,
                type_indicator,
                int, float,
                hardcoded,
            ))
            .map(|v| Value(v, PrintBehavior::Implicit));

            let explicit_print_values = choice((
                invert,
                explicit_print,
            ))
            .map(|v| Value(v, PrintBehavior::Explicit));

            let value = implicit_print_values.or(explicit_print_values).labelled("value");

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
            format!("{indent}// SCGT: {}\n{indent}{s}", &state.source[span.start..span.end])
        });

        statement
            .padded_by(just("\n").or_not())
            .repeated()
            .collect::<Vec<_>>()
            .map(|v| v.join("\n\n"))
    })
    .map_with_state(|mut code, _, state: &mut State| {
        if !state.variables.is_empty() {
            code = format!("{}\n{}{code}",
                "// Used variables",
                state.variables
                    .iter()
                    .rfold(String::from("\n"), |rest, var| {
                        format!("let {var} = null\n{rest}")
                    })
            );
        }

        if !state.helper_functions.is_empty() {
            code = format!("{}\n{}{code}",
                "// Automatically generated SCGT helper functions",
                state.helper_functions
                    .iter()
                    .rfold(String::from("\n"), |rest, helper| {
                        format!("{} = {}\n\n{rest}", helper.spwn_name(), helper.spwn_impl())
                    })
            );
        }

        code
    })
}
