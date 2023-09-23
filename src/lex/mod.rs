use logos::{Logos, Lexer};

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[regex(r"\d+\.\d+")]
    Float,

    #[regex(r"\d+")]
    Digits,

    #[regex(r"'.", |lex| lex.slice().chars().last().map(String::from))]
    #[regex(r"(`(\\.|[^`\n])*|\\(\\.|[^`])*)`", lex_string)]
    String(String),
    
    #[regex(r"[a-z]+", |lex| Some(lex.slice().to_string()))]
    #[regex(r"_\w+", |lex| Some(lex.slice()[1..].to_string()))]
    Ident(String),

    #[regex(r"@[^\d\W]\w*")]
    Type,

    #[token(".")] Dot,
    #[token(",")] Comma,
    #[token(":")] Colon,
    #[token("!")] Bang,
    #[token("?")] Question,
    #[token("$")] Dollar,

    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token("[")] LBracket,
    #[token("]")] RBracket,
    #[token("{")] LBrace,
    #[token("}")] RBrace,

    #[token(";")] Closing,
    
    #[token("A")] A,
    #[token("B")] B,
    #[token("C")] C,
    #[token("D")] D,
    #[token("E")] E,
    #[token("F")] F,
    #[token("G")] G,
    #[token("H")] H,
    #[token("I")] I,
    #[token("J")] J,
    #[token("K")] K,
    #[token("L")] L,
    #[token("M")] M,
    #[token("N")] N,
    #[token("O")] O,
    #[token("P")] P,
    #[token("Q")] Q,
    #[token("R")] R,
    #[token("S")] S,
    #[token("T")] T,
    #[token("U")] U,
    #[token("V")] V,
    #[token("W")] W,
    #[token("X")] X,
    #[token("Y")] Y,
    #[token("Z")] Z,
}

fn lex_string(lex: &mut Lexer<Token>) -> Option<String> {
    let slice = lex.slice();
    Some(slice[1..slice.len() - 1].to_string())
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! token_test {
        (
            $name:ident: $input:literal
            $( $t:expr, $span:expr, $val:literal )*
        ) => {
            #[test]
            fn $name() {
                let mut lex = Token::lexer($input);

                $(
                    assert_eq!(lex.next(), Some(Ok($t)));
                    assert_eq!(lex.span(), $span);
                    assert_eq!(lex.slice(), $val);
                )*

                assert_eq!(lex.next(), None);
            }
        }
    }

    token_test! {
        numbers: "3.14 1 5.92 6535"

        Token::Float,   0.. 4, "3.14"
        Token::Digits,  5.. 6, "1"
        Token::Float,   7..11, "5.92"
        Token::Digits, 12..16, "6535"
    }

    token_test! {
        strings: "`test` `esc\\ap\\\\es\\`` \\\newlines`"

        Token::String(String::from("test")),              0.. 6, "`test`"
        Token::String(String::from("esc\\ap\\\\es\\`")),  7..21, "`esc\\ap\\\\es\\``"
        Token::String(String::from("\newlines")),        22..32, "\\\newlines`"
    }

    token_test! {
        idents: "a bcde _T_3sT"

        Token::Ident(String::from("a")),     0.. 1, "a"
        Token::Ident(String::from("bcde")),  2.. 6, "bcde"
        Token::Ident(String::from("T_3sT")), 7..13, "_T_3sT"
    }

    token_test! {
        types: "@a @bcde @_T_3sT"

        Token::Type, 0.. 2, "@a"
        Token::Type, 3.. 8, "@bcde"
        Token::Type, 9..16, "@_T_3sT"
    }

    token_test! {
        keyletters: "ABC_DEF GHI"

        Token::A,                           0.. 1, "A"
        Token::B,                           1.. 2, "B"
        Token::C,                           2.. 3, "C"
        Token::Ident(String::from("DEF")),  3.. 7, "_DEF"
        Token::G,                           8.. 9, "G"
        Token::H,                           9..10, "H"
        Token::I,                          10..11, "I"
    }
}
