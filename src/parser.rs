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

    indent_size: usize,
    
    levels: Vec<Option<bool>>,
}

impl<'a> State<'a> {
    fn new(source: &'a str, indent_size: usize) -> Self {
        Self {
            helper_functions: BTreeSet::new(),
            variables: BTreeSet::new(),
            source,

            indent_size,

            levels: Vec::new(),
        }
    }

    fn add_helper(&mut self, helper: HelperFunction) -> &'static str {
        self.helper_functions.insert(helper);
        helper.spwn_name()
    }

    fn get_indent(&self) -> String {
        " ".repeat(self.indent_size)
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

pub fn parse(code: &str, indent_size: usize) -> ParseResult<String, Err<'_>> {
    let mut state = State::new(code, indent_size);
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
            let value = recursive(|value| {
                let int = text::int(10).map(String::from);

                let float = text::int(10).slice().or_not()
                    .then_ignore(just('.'))
                    .then(text::digits(10).slice().or_not())
                    .filter(|(bef, aft)| bef.as_ref().or(aft.as_ref()).is_some())
                    .map(|(bef, aft)| format!("{}.{}", bef.unwrap_or("0"), aft.unwrap_or("0")));
    
                let short_multiplication = int.or(float)
                    .then(value.clone())
                    .map_with_state(|(n, SpwnCode { code, .. }), _, state: &mut State| {
                        let helper = state.add_helper(HelperFunction::Mul);
                        format!("{helper}({n}, {code})")
                    });
    
                let char_literal = just('\'')
                    .ignore_then(any())
                    .map(|c| format!("\"{c}\""));
    
                let string_char = choice((
                    // SCGT escapes
                    just('\\')
                        .ignore_then(
                            select! {
                                'n' => "\\n",
                                'r' => "\\r",
                                't' => "\\t",
                                '`' => "`",
                                '\\' => "\\\\",
                            }
                        )
                        .map(String::from),
    
                    // SPWN escapes
                    select! {
                        '"' => "\\\"",
                        '\'' => "\\'",
                        '\\' => "\\\\",
                    }
                    .map(String::from),
    
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
                        let code = format_stmts(stmts, state, false, Some("return #"));
                        wrap_with_block(code, None)
                    });
    
                let invert = just('!')
                    .ignore_then(expression.clone())
                    .map_with_state(|SpwnCode { code, .. }, _, state: &mut State| {
                        let helper = state.add_helper(HelperFunction::Invert);
                        format!("{helper}({code})")
                    });
    
                let trigger_function = block.clone()
                    .delimited_by(just('}'), closing)
                    .map_with_state(|stmts, _, state: &mut State| {
                        // TODO check back here when `-> return`
                        let code = format_stmts(stmts, state, false, None);
                        format!("!{{\n{code}\n}}")
                    });

                let loop_variables = one_of("IJK")
                    .map_with_state(|name: char, _, state: &mut State| {
                        let name = format!("_scgt_loop_{}", name.to_lowercase());
                        state.variables.insert(name.clone());
                        name
                    });
    
                let macro_definition_no_args = block.clone()
                    .delimited_by(just('M'), closing)
                    .map_with_state(|stmts, _, state: &mut State| {
                        // TODO check back here when `-> return`
                        let code = format_stmts(stmts, state, false, Some("return #"));
                        format!("() {{\n{code}\n}}")
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
                    set_stmt(
                        choice((
                            short_multiplication,
                            int, float,
                            char_literal, string,
                            value_ident, type_indicator,
                            inner_block,
                            invert,
                            loop_variables,
                            macro_definition_no_args,
                            hardcoded,
                        )),
                        Some(false)
                    ),
                    set_stmt(
                        choice((
                            trigger_function,
                        )),
                        Some(true)
                    )
                ))
                .map_with_span(SpwnCode::implicit_print);
    
                let explicit_print = just('$')
                    .ignore_then(set_stmt(expression.clone(), Some(false)))
                    .map_with_state(|SpwnCode { code, .. }, _, state: &mut State| {
                        if state.is_stmt() {
                            format!("$.print({code})")
                        } else {
                            let helper = state.add_helper(HelperFunction::Print);
                            format!("{helper}({code})")
                        }
                    });

                let assignment = ident
                    .then(
                        set_stmt(expression.clone(), Some(false))
                            .delimited_by(just('!'), closing)
                    )
                    .map_with_state(|(name, SpwnCode { code, .. }), _, state: &mut State| {
                        let code = if state.is_stmt() {
                            format!("{name} = {code}")
                        } else {
                            let helper = state.add_helper(HelperFunction::Set);
                            format!("{helper}({name}, {code})")
                        };
                        state.variables.insert(name);
                        code
                    });
    
                let on_touch = expression.clone()
                    .delimited_by(just('E'), closing)
                    .map(|SpwnCode { code, .. }| {
                        format!("on(touch(), {code})")
                    });
    
                let infinite_loop = block.clone()
                    .delimited_by(just('L'), closing)
                    .map_with_state(|stmts, _, state| {
                        format_loop("while true", stmts, state)
                    });
    
                let explicit_print_values = choice((
                    // set_stmt called at definitions
                    explicit_print,
                    assignment,
                    set_stmt(on_touch, Some(false)),
                    set_stmt(infinite_loop, None),
                ))
                .map_with_span(SpwnCode::explicit_print);
    
                explicit_print_values.or(implicit_print_values).labelled("value")
            });

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
                let mut code = format_stmts(stmts, state, true, None);

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
                        "// Automatically generated helper functions",
                        state.helper_functions
                            .iter()
                            .rfold(String::new(), |rest, helper| {
                                let code = helper.spwn_impl().replace("    ", &state.get_indent());
                                format!("{} = {code}\n\n{rest}", helper.spwn_name())
                            })
                    );
                }

                format!("{code}\n")
            })
    ))
}

fn set_stmt<'a, T>(parser: parser_type!('a, T), is_stmt: Option<bool>) -> parser_type!('a, T) {
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
fn format_stmts(
    stmts: Vec<SpwnCode>,
    state: &mut State,
    global: bool,
    return_fmt: Option<&str>,
) -> String {
    if stmts.is_empty() {
        String::new()
    } else {
        let last_index = stmts.len() - 1;
        let indent = if global { String::new() } else { state.get_indent() };

        stmts
            .into_iter()
            .enumerate()
            .map(|(i, SpwnCode { code, span, print })| {
                let comment = state.source[span.start..span.end]
                    .lines()
                    .map(|line| format!("{indent}// {line}\n"))
                    .collect::<String>();

                let code = match return_fmt {
                    Some(r) if i == last_index => r.replace('#', &code),
                    _ => match print {
                        PrintBehavior::Explicit => code,
                        PrintBehavior::Implicit => format!("$.print({code})"),
                    }
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

fn wrap_with_block(mut code: String, indent: Option<String>) -> String {
    if let Some(indent) = indent {
        code = code
            .lines()
            .map(|line| format!("{indent}{line}"))
            .collect::<Vec<_>>()
            .join("\n");
    }

    format!("() {{\n{code}\n}} ()")
}

fn format_loop(start: &str, stmts: Vec<SpwnCode>, state: &mut State) -> String {
    let mut code;
    if stmts.is_empty() {
        code = format!("{start} {{ }}")
    } else if state.is_stmt() {
        code = format_stmts(stmts, state, false, None);
        code = format!("{start} {{\n{code}\n}}");
    } else {
        let arr_name = format!("_scgt_loop_{}", state.levels.len());
        code = format_stmts(stmts, state, false, Some(&format!("{arr_name}.push(#)")));
        code = format!("let {arr_name} = []\n{start} {{\n{code}\n}}\nreturn {arr_name}");
        state.variables.insert(arr_name);
    }

    if state.is_stmt() {
        code
    } else {
        wrap_with_block(code, Some(state.get_indent()))
    }
}
