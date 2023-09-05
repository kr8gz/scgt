use chumsky::prelude::*;

type ErrType<'a> = Rich<'a, char>;
type Err<'a> = extra::Err<ErrType<'a>>;

enum Modifier<'a> {
    AddDigits(&'a str),
    ConvertToID(char),
    SingleChars,
}

pub fn parse(code: &str) -> ParseResult<String, ErrType<'_>> {
    parser().parse(code)
}

fn parser<'a>() -> impl Parser<'a, &'a str, String, Err<'a>> + Clone {
    recursive(|parser| {
        let end = choice((end(), just(";").ignored(), just("\n").ignored()));

        let lowercase = one_of("abcdefghijklmnopqrstuvwxyz");

        let id = select! {
            'B' => 'b',
            'C' => 'c',
            'D' => 'i',
            'G' => 'g',
        };

        let num_mod = text::digits(10).slice().or_not().map(|opt| opt.map(Modifier::AddDigits));
        let id_mod = id.or_not().map(|opt| opt.map(Modifier::ConvertToID));
        // let char_list_mod = just(":").or_not().map(|opt| opt.map(|_| Modifier::SingleChars));

        let int = text::int(10).map(str::to_string);
        let float = text::int(10).slice().or_not()
            .then_ignore(just("."))
            .then(text::digits(10).slice().or_not())
            .filter(|(bef, aft)| bef.as_ref().or(aft.as_ref()).is_some())
            .map(|(bef, aft)| format!("{}.{}", bef.unwrap_or("0"), aft.unwrap_or("0")));

        let id_value = text::int(10).slice().or(just("?"))
            .then(id)
            .map(|(int, id)| format!("{int}{id}"));

        let ident = lowercase.repeated().at_least(1)
            .collect::<String>()
            .or(just("_").ignore_then(text::ident()).map(str::to_string))
            .labelled("identifier");

        let trigger_fn_def = parser.clone().delimited_by(just("}"), end)
            .map(|statements| format!("!{{{statements}}}"));

        let character = just("'").ignore_then(any()).map(|c: char| c.to_string())
            .map(|c| format!("'{c}'"));

        let value = choice((
            trigger_fn_def,
            id_value,
            float,
            int,
            character,
            just("T").to("true".to_string()),
            just("F").to("false".to_string()),
            just("S").to("\"\"".to_string()),
            ident.or(just("$").map(str::to_string)),
        ))
        .foldl(
            // choice((
                just(".").ignore_then(ident).map(|member| format!(".{member}"))
            // ))
            .repeated(),
            |parent, op| format!("{parent}{op}")
        )
        .labelled("value");

        let implicit_print_expression = num_mod.then(id_mod).then(ident)
        .map(|((num_mod, id_mod), mut expr)| {
            if let Some(Modifier::AddDigits(num)) = num_mod {
                expr = format!("@string({expr}) + \"{num}\"")
            }
            if let Some(Modifier::ConvertToID(id)) = id_mod {
                expr = match id {
                    'g' => format!("@group(@number({expr}))"),
                    'c' => format!("@color(@number({expr}))"),
                    'i' => format!("@item(@number({expr}))"),
                    'b' => format!("@block(@number({expr}))"),
                    _ => unreachable!()
                }
            }
            expr
        });

        let expression = value.clone() // .foldl(
            // choice((
                // put SOMETHING here (e.g. macro call) so that not every value is an expression
            // ))
            // .repeated().at_least(1),
            // |parent, op| format!("{parent}{op}")
        // )
        .labelled("expression");

        let assign = ident.then(expression.clone().or(value.clone()))
            .map(|(var, expr)| format!("let {var} = {expr}"));

        let trigger_fn_call = text::int(10).slice().or(just("?")).then_ignore(just("!")).map(|id| format!("{id}g!"))
            .or(expression.clone().then_ignore(just("!")).map(|expr| format!("{expr}!")));

        let explicit_print = just("$").ignore_then(just(".").not()).ignore_then(expression.clone())
            .map(|expr| format!("$.print({expr})")); // check for no following `.` in case .x syntax is added

        let statement = choice((
            assign,
            trigger_fn_call,
            explicit_print,
            // expression, // TODO uncomment when proper expressions are added
            implicit_print_expression.or(value).map(|value| format!("$.print({value})")),
        ))
        .labelled("statement");

        statement
            .padded_by(just("\n").or_not())
            .repeated()
            .collect::<Vec<_>>()
            .map(|v| v.join("\n"))
    })
}
