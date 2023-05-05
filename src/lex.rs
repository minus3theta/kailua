use combine::{
    between, choice, eof, many, many1,
    parser::char::{letter, spaces},
    satisfy, token, ParseError, Parser, Stream,
};

#[derive(Debug)]
pub enum Token {
    Name(String),
    String(String),
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
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let name = many1(letter()).map(Token::Name);
    let string = between(token('"'), token('"'), many(satisfy(|c| c != '"'))).map(Token::String);
    let eos = eof().map(|_| Token::Eos);
    spaces().with(choice((name, string, eos)))
}
