use std::collections::HashSet;

use chumsky::prelude::*;

use crate::helper_functions::*;

type Err<'a> = Rich<'a, char>;
type Extra<'a> = extra::Full<Err<'a>, State, ()>;

struct State {
    helper_functions: HashSet<HelperFunction>,
}

impl State {
    fn new() -> Self {
        State {
            helper_functions: HashSet::new(),
        }
    }

    fn add_helper(&mut self, helper: HelperFunction) -> &'static str {
        self.helper_functions.insert(helper);
        helper.spwn_name()
    }
}

pub fn parse(code: &str) -> ParseResult<String, Err<'_>> {
    let mut state = State::new();
    parser().parse_with_state(code, &mut state)
}

fn parser<'a>() -> impl Parser<'a, &'a str, String, Extra<'a>> + Clone {
    recursive(|block| {
        let ident = one_of("abcdefghijklmnopqrstuvwxyz").repeated().at_least(1)
            .collect::<String>()
            .or(just("_").ignore_then(text::ident()).map(str::to_string))
            .labelled("identifier");

        let value = recursive(|value| {
            let int = text::int(10).map(str::to_string);

            let float = text::int(10).slice().or_not()
                .then_ignore(just("."))
                .then(text::digits(10).slice().or_not())
                .filter(|(bef, aft)| bef.as_ref().or(aft.as_ref()).is_some())
                .map(|(bef, aft)| format!("{}.{}", bef.unwrap_or("0"), aft.unwrap_or("0")));

            let invert = just("!") // TODO non-implicit print
                .ignore_then(value.clone())
                .map_with_state(|v, _, state: &mut State| {
                    let helper = state.add_helper(HelperFunction::Invert);
                    format!("{helper}({v})")
                });

            let explicit_print = just("$") // TODO non-implicit print
                .ignore_then(value.clone())
                .map_with_state(|v, _, state: &mut State| {
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

            choice((
                ident,
                int, float,
                invert,
                explicit_print,
                hardcoded,
            ))
        })
        .labelled("value");

        // let expression = 
        // TODO how to handle implicit/explicit printing? state?

        value // statement
            .padded_by(just("\n").or_not())
            .repeated()
            .collect::<Vec<_>>()
            .map(|v| v.join("\n"))
    })
    .map_with_state(|code, _, state: &mut State| {
        if state.helper_functions.is_empty() {
            code
        } else {
            format!("{}\n{}",
                "// Automatically generated SCGT helper functions",
                state.helper_functions
                    .iter()
                    .fold(code, |code, helper| {
                        format!("{} = {}\n{}", helper.spwn_name(), helper.spwn_impl(), code)
                    })
            )
        }
    })
}
