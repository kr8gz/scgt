/// throughout this file '#' will be used as a placeholder for generated code to be inserted

use std::collections::BTreeSet;

use chumsky::prelude::*;

use crate::helpers::*;

type Err<'a> = Rich<'a, char>;
type Extra<'a> = extra::Full<Err<'a>, State<'a>, ()>;

macro_rules! parser_type {
    ( $lt:lifetime, $ret_t:ty ) => {
        impl Parser<$lt, &$lt str, $ret_t, Extra<$lt>> + Clone
    }
}

struct State<'a> {
    helpers: BTreeSet<HelperFunction>,
    variables: BTreeSet<String>,
    source: &'a str,

    indent_size: usize,
    
    depth: usize,
}

impl<'a> State<'a> {
    fn new(source: &'a str, indent_size: usize) -> Self {
        Self {
            helpers: BTreeSet::new(),
            variables: BTreeSet::new(),
            source,

            indent_size,

            depth: 1,
        }
    }

    fn add_helper(&mut self, helper: HelperFunction) -> &'static str {
        self.helpers.insert(helper);
        helper.spwn_name()
    }

    fn get_indent(&self) -> String {
        " ".repeat(self.indent_size)
    }
}

enum PrintBehavior {
    Implicit,
    Explicit,
}

struct CodeVariables {
    code: String,
    helpers: Option<BTreeSet<HelperFunction>>,
    variables: Option<BTreeSet<String>>,
}

impl CodeVariables {
    fn none(code: String) -> Self {
        Self {
            code,
            helpers: None,
            variables: None,
        }
    }
}

struct SpwnCode {
    expr: CodeVariables,
    stmt: Option<CodeVariables>,
    span: SimpleSpan,
    print: PrintBehavior,
}

impl SpwnCode {
    fn simple_implicit(code: CodeVariables, span: SimpleSpan) -> Self {
        SpwnCode {
            expr: code, stmt: None,
            span,
            print: PrintBehavior::Implicit,
        }
    }

    fn simple_explicit(code: CodeVariables, span: SimpleSpan) -> Self {
        SpwnCode {
            expr: code, stmt: None,
            span,
            print: PrintBehavior::Explicit,
        }
    }

    fn get_code(&self, is_stmt: bool, state: &mut State) -> String {
        let code = if is_stmt {
            self.stmt.as_ref().unwrap_or(&self.expr)
        } else {
            &self.expr
        };

        if let Some(helpers) = &code.helpers {
            state.helpers.extend(helpers.iter());
        }

        if let Some(variables) = &code.variables {
            state.variables.extend(variables.iter().cloned());
        }

        code.code.to_string()
    }
}

pub fn parse(code: &str, indent_size: usize) -> ParseResult<String, Err<'_>> {
    let mut state = State::new(code, indent_size);
    parser().parse_with_state(code, &mut state)
}

fn parser<'a>() -> parser_type!('a, String) {
    let global = recursive(|block| {
        let block = empty()
            .map_with_state(move |_, _, state: &mut State| {
                state.depth += 1;
            })
            .then(block)
            // set to previous state in ALL CASES
            .map_with_state(|((), out), _, state: &mut State| {
                state.depth -= 1;
                out
            })
            .map_err_with_state(|err, _, state: &mut State| {
                state.depth -= 1;
                err
            });

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
                    .then_ignore(none_of("ABCDEFGLMNOSWX").rewind())
                    .then(value.clone())
                    .map_with_state(|(n, code): (String, SpwnCode), _, state: &mut State| {
                        let helper = state.add_helper(HelperFunction::Mul);
                        format!("{helper}({n}, {})", code.get_code(false, state))
                    });
    
                let char_literal = just('\'')
                    .ignore_then(any())
                    .map(|c: char| format!("\"{c}\""));
    
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
                .map(|v: Vec<String>| format!("\"{}\"", v.into_iter().collect::<String>()));
    
                let value_ident = ident
                    .map_with_state(|name: String, _, state: &mut State| {
                        let helper = state.add_helper(HelperFunction::Get);
                        let code = format!("{helper}({name})");
                        state.variables.insert(name);
                        code
                    });
    
                let type_indicator = just('@')
                    .ignore_then(ident)
                    .map(|name: String| format!("@{name}"));
    
                let inner_block = block.clone()
                    .delimited_by(just('('), closing)
                    .map_with_state(|stmts: Vec<SpwnCode>, _, state: &mut State| {
                        let code = format_stmts(&stmts, state, false, Some("return #"));
                        wrap_with_block(code, None)
                    });
    
                let invert = just('!')
                    .ignore_then(expression.clone())
                    .map_with_state(|code: SpwnCode, _, state: &mut State| {
                        let helper = state.add_helper(HelperFunction::Invert);
                        format!("{helper}({})", code.get_code(false, state))
                    });
    
                let trigger_function = block.clone()
                    .delimited_by(just('}'), closing)
                    .map_with_state(|stmts: Vec<SpwnCode>, _, state: &mut State| {
                        // TODO check back here when `-> return`
                        let code = format_stmts(&stmts, state, false, None);
                        format!("!{{\n{code}\n}}")
                    });

                let loop_variables = one_of("IJK")
                    .map_with_state(|name: char, _, state: &mut State| {
                        let name = format!("_scgt_loop_{}", name.to_lowercase());
                        state.variables.insert(name.clone());
                        name
                    });
    
                let macro_def_no_args = block.clone()
                    .delimited_by(just('M'), closing)
                    .map_with_state(|stmts: Vec<SpwnCode>, _, state: &mut State| {
                        let code = format_stmts(&stmts, state, false, Some("return #"));
                        format!("() {{\n{code}\n}}")
                    });
    
                let macro_def_x_arg = block.clone()
                    .delimited_by(just('X'), closing)
                    .map_with_state(|stmts: Vec<SpwnCode>, _, state: &mut State| {
                        let code = format_stmts(&stmts, state, false, Some("return #"));
                        format!("(x) {{\n{code}\n}}")
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
                    short_multiplication,
                    int, float,
                    char_literal, string,
                    value_ident, type_indicator,
                    inner_block,
                    invert,
                    loop_variables,
                    macro_def_no_args.clone(), macro_def_x_arg.clone(), // named shortcuts in explicit print section
                    hardcoded,
                    trigger_function,
                ))
                // simple_implicit guarantees that the required variables and helper functions will be added
                .map(CodeVariables::none)
                .map_with_span(SpwnCode::simple_implicit);
    
                let explicit_print = just('$')
                    .ignore_then(expression.clone())
                    .map_with_state(|code: SpwnCode, span, state: &mut State| {
                        let code = code.get_code(false, state);

                        let mut helpers = BTreeSet::new();
                        let print = HelperFunction::Print;
                        helpers.insert(print);

                        SpwnCode {
                            expr: CodeVariables {
                                code: format!("{}({code})", print.spwn_name()),
                                helpers: Some(helpers),
                                variables: None,
                            },
                            stmt: Some(CodeVariables::none(format!("$.print({code})"))),
                            span,
                            print: PrintBehavior::Explicit,
                        }
                    });
    
                let on_touch = expression.clone()
                    .delimited_by(just('E'), closing)
                    .map_with_state(|code: SpwnCode, _, state: &mut State| {
                        format!("on(touch(), {})", code.get_code(false, state))
                    })
                    .map(CodeVariables::none)
                    .map_with_span(SpwnCode::simple_explicit);
    
                let infinite_loop = block.clone()
                    .delimited_by(just('L'), closing)
                    .map_with_state(|stmts: Vec<SpwnCode>, span, state: &mut State| {
                        format_loop("while true", &stmts, span, state)
                    });

                let named_macro_no_args = ident.then(macro_def_no_args)
                    .map_with_span(|(name, code): (String, String), span| {
                        format_assign(name, code, span)
                    });

                let named_macro_x_arg = ident.then(macro_def_x_arg)
                    .map_with_span(|(name, code): (String, String), span| {
                        format_assign(name, code, span)
                    });
    
                let atom = choice((
                    explicit_print,
                    on_touch,
                    infinite_loop,
                    named_macro_no_args, named_macro_x_arg,
                    implicit_print_values,
                ));

                struct Postfix {
                    span: SimpleSpan,
                    data: PostfixType,
                }

                enum PostfixType {
                    Assignment { expr: String },
                    MemberAccess { name: String },
                    MacroCallNoArgs,
                }

                let assignment = expression.clone().delimited_by(just('!'), closing)
                    .map_with_state(|code: SpwnCode, span, state: &mut State| {
                        Postfix {
                            span,
                            data: PostfixType::Assignment {
                                expr: code.get_code(false, state),
                            },
                        }
                    });

                let member_access = just('.')
                    .ignore_then(ident)
                    .map_with_span(|name: String, span| {
                        Postfix {
                            span,
                            data: PostfixType::MemberAccess { name },
                        }
                    });

                let macro_call_no_args = just('M').ignored()
                    .map_with_span(|_, span| {
                        Postfix {
                            span,
                            data: PostfixType::MacroCallNoArgs,
                        }
                    });

                let postfixes = atom.foldl_with_state(
                    choice((
                        assignment,
                        member_access,
                        macro_call_no_args,
                    ))
                    .repeated(),
                    |value: SpwnCode, postfix: Postfix, state: &mut State| {
                        let code = value.get_code(false, state);
                        let span = (value.span.start..postfix.span.end).into();

                        match postfix.data {
                            PostfixType::Assignment { expr } => {
                                format_assign(code, expr, span)
                            }
                            
                            PostfixType::MemberAccess { name } => {
                                SpwnCode::simple_implicit(
                                    CodeVariables::none(format!("{code}.{name}")),
                                    span,
                                )
                            }
                            
                            PostfixType::MacroCallNoArgs => {
                                let mut helpers = BTreeSet::new();
                                let call = HelperFunction::Call;
                                helpers.insert(call);

                                SpwnCode::simple_explicit(
                                    CodeVariables {
                                        code: format!("{}({code})", call.spwn_name()),
                                        helpers: Some(helpers),
                                        variables: None,
                                    },
                                    span,
                                )
                            }
                        }
                    },
                );

                postfixes
                    .labelled("value")
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
            .map_with_state(|stmts: Vec<SpwnCode>, _, state: &mut State| {
                let mut code = format_stmts(&stmts, state, true, None);

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

                if !state.helpers.is_empty() {
                    code = format!("{}\n{}{code}",
                        "// Automatically generated helper functions",
                        state.helpers
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

fn format_stmts(
    stmts: &Vec<SpwnCode>,
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
            .iter()
            .enumerate()
            .map(|(i, code)| {
                let comment = state.source[code.span.start..code.span.end]
                    .lines()
                    .map(|line| format!("{indent}// {line}\n"))
                    .collect::<String>();

                let code = match return_fmt {
                    Some(r) if i == last_index => r.replace('#', &code.get_code(false, state)),
                    _ => match code.print {
                        PrintBehavior::Explicit => code.get_code(true, state).to_string(),
                        PrintBehavior::Implicit => format!("$.print({})", code.get_code(false, state)),
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

fn format_loop(start: &str, stmts: &Vec<SpwnCode>, span: SimpleSpan, state: &mut State) -> SpwnCode {
    let expr;
    let stmt;

    if stmts.is_empty() {
        let code = format!("{start} {{ }}");
        stmt = CodeVariables::none(code.clone());
        expr = CodeVariables::none(wrap_with_block(code, Some(state.get_indent())))
    } else {
        stmt = CodeVariables::none(format!("{start} {{\n{}\n}}", format_stmts(stmts, state, false, None)));

        let arr_name = format!("_scgt_loop_{}", state.depth);
        let mut code = format_stmts(stmts, state, false, Some(&format!("{arr_name}.push(#)")));
        code = format!("let {arr_name} = []\n{start} {{\n{code}\n}}\nreturn {arr_name}");
        code = wrap_with_block(code, Some(state.get_indent()));

        let mut variables = BTreeSet::new();
        variables.insert(arr_name);
        expr = CodeVariables { code, helpers: None, variables: Some(variables) };
    }

    SpwnCode {
        expr, stmt: Some(stmt),
        span,
        print: PrintBehavior::Explicit,
    }
}

fn format_assign(target: String, value: String, span: SimpleSpan) -> SpwnCode {
    let mut helpers = BTreeSet::new();
    let set = HelperFunction::Set;
    helpers.insert(set);

    let variables = text::ident::<_, _, extra::Default>()
        .parse(target.as_str())
        .output()
        .map(|target| {
            let mut variables = BTreeSet::new();
            variables.insert(target.to_string());
            variables
        });

    SpwnCode {
        expr: CodeVariables {
            code: format!("{}({target}, {value})", set.spwn_name()),
            helpers: Some(helpers),
            variables: variables.clone(),
        },
        stmt: Some(CodeVariables {
            code: format!("{target} = {value}"),
            helpers: None,
            variables,
        }),
        span,
        print: PrintBehavior::Explicit,
    }
}
