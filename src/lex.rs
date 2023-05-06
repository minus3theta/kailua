use combine::{
    attempt, between, choice, eof, from_str, many, many1,
    parser::{
        char::{digit, letter, spaces, string},
        combinator::recognize,
    },
    satisfy, token, Parser, Stream,
};

#[derive(Debug, PartialEq)]
pub enum Token {
    // keywords
    And,
    Break,
    Do,
    Else,
    Elseif,
    End,
    False,
    For,
    Function,
    Goto,
    If,
    In,
    Local,
    Nil,
    Not,
    Or,
    Repeat,
    Return,
    Then,
    True,
    Until,
    While,

    // operators
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Len,
    BitAnd,
    BitXor,
    BitOr,
    ShiftL,
    ShiftR,
    Idiv,
    Equal,
    NotEq,
    LesEq,
    GreEq,
    Less,
    Greater,
    Assign,
    ParL,
    ParR,
    CurlyL,
    CurlyR,
    SqurL,
    SqurR,
    DoubColon,
    SemiColon,
    Colon,
    Comma,
    Dot,
    Concat,
    Dots,

    // constant values
    Integer(i64),
    Float(f64),
    String(String),

    // name of variables or table keys
    Name(String),

    // end
    Eos,
}

#[derive(Debug)]
pub struct Lex<'a> {
    input: &'a str,
}

impl<'a> Lex<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input }
    }

    pub fn next(&mut self) -> anyhow::Result<Token> {
        let (t, rest) = lua_token().parse(self.input)?;
        self.input = rest;
        Ok(t)
    }
}

fn lua_token<Input>() -> impl Parser<Input, Output = Token>
where
    Input: Stream<Token = char>,
{
    let name = many1(letter()).map(Token::Name);
    let string = between(token('"'), token('"'), many(satisfy(|c| c != '"'))).map(Token::String);
    let eos = eof().map(|_| Token::Eos);
    spaces().with(choice((
        keywords(),
        operators(),
        attempt(float()),
        integer(),
        name,
        string,
        eos,
    )))
}

fn keywords<Input>() -> impl Parser<Input, Output = Token>
where
    Input: Stream<Token = char>,
{
    choice((
        attempt(string("and")).map(|_| Token::And),
        attempt(string("break")).map(|_| Token::Break),
        attempt(string("do")).map(|_| Token::Do),
        attempt(string("elseif")).map(|_| Token::Elseif),
        attempt(string("else")).map(|_| Token::Else),
        attempt(string("end")).map(|_| Token::End),
        attempt(string("false")).map(|_| Token::False),
        attempt(string("for")).map(|_| Token::For),
        attempt(string("function")).map(|_| Token::Function),
        attempt(string("goto")).map(|_| Token::Goto),
        attempt(string("if")).map(|_| Token::If),
        attempt(string("in")).map(|_| Token::In),
        attempt(string("local")).map(|_| Token::Local),
        attempt(string("nil")).map(|_| Token::Nil),
        attempt(string("not")).map(|_| Token::Not),
        attempt(string("or")).map(|_| Token::Or),
        attempt(string("repeat")).map(|_| Token::Repeat),
        attempt(string("return")).map(|_| Token::Return),
        attempt(string("then")).map(|_| Token::Then),
        attempt(string("true")).map(|_| Token::True),
        attempt(string("until")).map(|_| Token::Until),
        attempt(string("while")).map(|_| Token::While),
    ))
}

fn operators<Input>() -> impl Parser<Input, Output = Token>
where
    Input: Stream<Token = char>,
{
    choice((
        attempt(string("...")).map(|_| Token::Dots),
        choice((
            attempt(string("<<")).map(|_| Token::ShiftL),
            attempt(string(">>")).map(|_| Token::ShiftR),
            attempt(string("//")).map(|_| Token::Idiv),
            attempt(string("==")).map(|_| Token::Equal),
            attempt(string("~=")).map(|_| Token::NotEq),
            attempt(string("<=")).map(|_| Token::LesEq),
            attempt(string(">=")).map(|_| Token::GreEq),
            attempt(string("::")).map(|_| Token::DoubColon),
            attempt(string("..")).map(|_| Token::Concat),
        )),
        choice((
            attempt(string("+")).map(|_| Token::Add),
            attempt(string("-")).map(|_| Token::Sub),
            attempt(string("*")).map(|_| Token::Mul),
            attempt(string("/")).map(|_| Token::Div),
            attempt(string("%")).map(|_| Token::Mod),
            attempt(string("^")).map(|_| Token::Pow),
            attempt(string("#")).map(|_| Token::Len),
            attempt(string("&")).map(|_| Token::BitAnd),
            attempt(string("~")).map(|_| Token::BitXor),
            attempt(string("|")).map(|_| Token::BitOr),
            attempt(string("<")).map(|_| Token::Less),
            attempt(string(">")).map(|_| Token::Greater),
            attempt(string("=")).map(|_| Token::Assign),
            attempt(string("(")).map(|_| Token::ParL),
            attempt(string(")")).map(|_| Token::ParR),
            attempt(string("{")).map(|_| Token::CurlyL),
            attempt(string("}")).map(|_| Token::CurlyR),
            attempt(string("[")).map(|_| Token::SqurL),
            attempt(string("]")).map(|_| Token::SqurR),
            attempt(string(";")).map(|_| Token::SemiColon),
            attempt(string(":")).map(|_| Token::Colon),
            attempt(string(",")).map(|_| Token::Comma),
            attempt(string(".")).map(|_| Token::Dot),
        )),
    ))
}

fn integer<Input>() -> impl Parser<Input, Output = Token>
where
    Input: Stream<Token = char>,
{
    from_str(many1::<String, _, _>(digit())).map(Token::Integer)
}

fn float<Input>() -> impl Parser<Input, Output = Token>
where
    Input: Stream<Token = char>,
{
    from_str(recognize::<String, _, _>((
        many1::<String, _, _>(digit()),
        token('.'),
        many1::<String, _, _>(digit()),
    )))
    .map(Token::Float)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_elseif() {
        let (tok, rest) = lua_token().parse("elseif").unwrap();
        assert_eq!(tok, Token::Elseif);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_idiv() {
        let (tok, rest) = lua_token().parse("//").unwrap();
        assert_eq!(tok, Token::Idiv);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_dots() {
        let (tok, rest) = lua_token().parse("...").unwrap();
        assert_eq!(tok, Token::Dots);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_integer() {
        let (tok, rest) = lua_token().parse("123").unwrap();
        assert_eq!(tok, Token::Integer(123));
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_float() {
        let (tok, rest) = lua_token().parse("123.45").unwrap();
        assert_eq!(tok, Token::Float(123.45));
        assert!(rest.is_empty());
    }
}
