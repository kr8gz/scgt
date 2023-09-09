use std::collections::BTreeSet;

use chumsky::prelude::*;

use crate::helper_functions::*;

type Err<'a> = Rich<'a, char>;
type Extra<'a> = extra::Full<Err<'a>, State<'a>, ()>;

macro_rules! parser_type {
    ( $lt:lifetime, $ret_t:ty ) => {
        impl Parser<$lt, &$lt str, $ret_t, Extra<$lt>> + Clone
    }
}

struct State<'a> {
    helper_functions: BTreeSet<HelperFunction>,
    variables: BTreeSet<String>,
    source: &'a str,
    
    levels: Vec<Option<bool>>,
}

impl<'a> State<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            helper_functions: BTreeSet::new(),
            variables: BTreeSet::new(),
            source,

            levels: Vec::new(),
        }
    }

    fn add_helper(&mut self, helper: HelperFunction) -> &'static str {
        self.helper_functions.insert(helper);
        helper.spwn_name()
    }

    fn is_stmt(&self) -> bool {
        self.levels
            .iter()
            .rev()
            .find_map(Option::as_ref)
            .copied()
            .unwrap_or(true)
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

fn parser<'a>() -> parser_type!('a, String) {
    let global = recursive(|block| {
        let ident = one_of("abcdefghijklmnopqrstuvwxyz")
            .repeated().at_least(1)
            .collect::<String>()
            .or(
                just('_')
                    .ignore_then(text::ident())
                    // .filter(|name: &&str| !name.starts_with("_scgt_"))
                    // could also validate with warning or something
                    .map(String::from)
            )
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
                    .map(|s| s.unwrap_or("\\").to_string()),

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

            let value_ident = ident.map_with_state(|name, _, state: &mut State| {
                let helper = state.add_helper(HelperFunction::Get);
                let code = format!("{helper}({name})");
                state.variables.insert(name);
                code
            });

            let type_indicator = just('@')
                .ignore_then(ident)
                .map(|name| format!("@{name}"));

            let inner_block = block.clone()
                .delimited_by(just('('), closing)
                .map_with_state(|stmts, _, state: &mut State| {
                    let code = format_stmts(stmts, state, false, "return #");
                    format!("() {{\n{code}\n}} ()")
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

            let implicit_print_values = set_stmt(
                choice((
                    int, float,
                    char_literal, string,
                    value_ident, type_indicator,
                    inner_block,
                    invert,
                    hardcoded,
                )),
                Some(false)
            )
            .map_with_span(SpwnCode::implicit_print);

            let explicit_print = just('$')
                .ignore_then(expression.clone())
                .map_with_state(|SpwnCode { code, .. }, _, state: &mut State| {
                    let helper = state.add_helper(HelperFunction::Print);
                    format!("{helper}({code})")
                });

            let infinite_loop = block.clone()
                .delimited_by(just('L'), closing)
                .map_with_state(|stmts, _, state| {
                    format_loop("while true", "while", stmts, state)
                });

            let explicit_print_values = choice((
                set_stmt(explicit_print, Some(false)),
                set_stmt(infinite_loop, None),
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
        .separated_by(just('\n').repeated())
        .allow_leading()
        .allow_trailing()
        .collect::<Vec<_>>()
        .labelled("statement")
    });

    choice((
        just("SCGT").map(|_| {
            let _ = open::that("https://github.com/kr8gz/scgt/");
            ":)".to_string()
        }),

        global
            .map_with_state(|stmts, _, state: &mut State| {
                let mut code = format_stmts(stmts, state, true, "");

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

                format!("{code}\n")
            })
    ))
}

fn set_stmt<'a>(parser: parser_type!('a, String), is_stmt: Option<bool>) -> parser_type!('a, String) {
    empty()
        .map_with_state(move |_, _, state: &mut State| {
            state.levels.push(is_stmt)
        })
        .then(parser)
        // set to previous state in ALL CASES
        .map_err_with_state(|err, _, state: &mut State| {
            state.levels.pop();
            err
        })
        .map_with_state(|((), code), _, state: &mut State| {
            state.levels.pop();
            code
        })
}

/// `return_fmt` replaces `#` with last value
fn format_stmts(stmts: Vec<SpwnCode>, state: &mut State, global: bool, return_fmt: &str) -> String {
    if stmts.is_empty() {
        String::new()
    } else {
        let last_index = stmts.len() - 1;
        let indent = if global { "" } else { "    " };

        stmts
            .into_iter()
            .enumerate()
            .map(|(i, SpwnCode { code, span, print })| {
                let comment = state.source[span.start..span.end]
                    .lines()
                    .map(|line| format!("{indent}// {line}\n"))
                    .collect::<String>();

                let code = if i < last_index || state.is_stmt() {
                    match print {
                        PrintBehavior::Explicit => code,
                        PrintBehavior::Implicit => {
                            if state.is_stmt() {
                                format!("$.print({code})")
                            } else {
                                let helper = state.add_helper(HelperFunction::Print);
                                format!("{helper}({code})")
                            }
                        }
                    }
                } else {
                    return_fmt.replace('#', &code)
                }
                .lines()
                .map(|line| format!("{indent}{line}"))
                .collect::<Vec<_>>()
                .join("\n");
            
                format!("{comment}{code}")
            })
            .collect::<Vec<_>>()
            .join(if global { "\n\n" } else { "\n" })
    }
}

fn wrap_with_block(mut code: String) -> String {
    code = code
        .lines()
        .map(|line| format!("    {line}\n"))
        .collect::<String>();

    format!("() {{\n{code}}} ()")
}

fn format_loop(start: &str, loop_type: &str, stmts: Vec<SpwnCode>, state: &mut State) -> String {
    if state.is_stmt() {
        let code = format_stmts(stmts, state, false, "");
        format!("{start} {{\n{code}\n}}")
    } else {
        let arr_name = format!("_scgt_{loop_type}_{}", state.levels.len());
        let mut code = format_stmts(
            stmts, state, false,
            &format!("{arr_name}.push(#)")
        );

        code = wrap_with_block(format!(
            "let {arr_name} = []\n{start} {{\n{code}\n}}\nreturn a"
        ));
        
        state.variables.insert(arr_name);
        code
    }
}
