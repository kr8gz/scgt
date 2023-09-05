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
    end().to(String::new())
}
