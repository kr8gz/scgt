use chumsky::prelude::*;

type ErrType<'a> = Rich<'a, char>;
type Err<'a> = extra::Err<ErrType<'a>>;

pub fn parse(code: &str) -> ParseResult<String, ErrType<'_>> {
    parser().parse(code)
}

fn parser<'a>() -> impl Parser<'a, &'a str, String, Err<'a>> + Clone {
    end().to(String::new())
}
