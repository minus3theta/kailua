use combine::{
    attempt, between, choice, eof, from_str, many, many1,
    parser::{
        byte::{bytes, digit, letter, spaces},
        combinator::recognize,
    },
    satisfy, token, Parser, Stream,
};

pub trait ByteStream<'a>: Stream<Token = u8, Range = &'a [u8]> + 'a {}
impl<'a, T: Stream<Token = u8, Range = &'a [u8]> + 'a> ByteStream<'a> for T {}

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

pub struct Lex<S> {
    input: Option<S>,
    ahead: Token,
}

impl<'a, S: ByteStream<'a>> Lex<S> {
    pub fn new(input: S) -> Self {
        Self {
            input: Some(input),
            ahead: Token::Eos,
        }
    }

    pub fn next(&mut self) -> anyhow::Result<Token> {
        if self.ahead == Token::Eos {
            self.do_next()
        } else {
            Ok(std::mem::replace(&mut self.ahead, Token::Eos))
        }
    }

    pub fn peek(&mut self) -> anyhow::Result<&Token> {
        if self.ahead == Token::Eos {
            self.ahead = self.do_next()?;
        }
        Ok(&self.ahead)
    }

    fn do_next(&mut self) -> anyhow::Result<Token> {
        let input = self.input.take();
        let (t, rest) = lua_token()
            .parse(input.unwrap())
            .map_err(|_| anyhow::anyhow!("parse failed"))?;
        self.input = Some(rest);
        Ok(t)
    }
}

fn lua_token<'a, Input>() -> impl Parser<Input, Output = Token> + 'a
where
    Input: ByteStream<'a>,
{
    let name = recognize((letter(), many::<Vec<_>, _, _>(letter().or(digit()))))
        .map(|v: Vec<u8>| Token::Name(String::from_utf8_lossy(&v).to_string()));
    let string = between(token(b'"'), token(b'"'), many(satisfy(|c| c != b'"')))
        .map(|v: Vec<u8>| Token::String(String::from_utf8_lossy(&v).to_string()));
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

fn keywords<'a, Input>() -> impl Parser<Input, Output = Token> + 'a
where
    Input: ByteStream<'a>,
{
    choice((
        attempt(bytes(&b"and"[..])).map(|_| Token::And),
        attempt(bytes(&b"break"[..])).map(|_| Token::Break),
        attempt(bytes(&b"do"[..])).map(|_| Token::Do),
        attempt(bytes(&b"elseif"[..])).map(|_| Token::Elseif),
        attempt(bytes(&b"else"[..])).map(|_| Token::Else),
        attempt(bytes(&b"end"[..])).map(|_| Token::End),
        attempt(bytes(&b"false"[..])).map(|_| Token::False),
        attempt(bytes(&b"for"[..])).map(|_| Token::For),
        attempt(bytes(&b"function"[..])).map(|_| Token::Function),
        attempt(bytes(&b"goto"[..])).map(|_| Token::Goto),
        attempt(bytes(&b"if"[..])).map(|_| Token::If),
        attempt(bytes(&b"in"[..])).map(|_| Token::In),
        attempt(bytes(&b"local"[..])).map(|_| Token::Local),
        attempt(bytes(&b"nil"[..])).map(|_| Token::Nil),
        attempt(bytes(&b"not"[..])).map(|_| Token::Not),
        attempt(bytes(&b"or"[..])).map(|_| Token::Or),
        attempt(bytes(&b"repeat"[..])).map(|_| Token::Repeat),
        attempt(bytes(&b"return"[..])).map(|_| Token::Return),
        attempt(bytes(&b"then"[..])).map(|_| Token::Then),
        attempt(bytes(&b"true"[..])).map(|_| Token::True),
        attempt(bytes(&b"until"[..])).map(|_| Token::Until),
        attempt(bytes(&b"while"[..])).map(|_| Token::While),
    ))
}

fn operators<'a, Input>() -> impl Parser<Input, Output = Token> + 'a
where
    Input: ByteStream<'a>,
{
    choice((
        attempt(bytes(&b"..."[..])).map(|_| Token::Dots),
        choice((
            attempt(bytes(&b"<<"[..])).map(|_| Token::ShiftL),
            attempt(bytes(&b">>"[..])).map(|_| Token::ShiftR),
            attempt(bytes(&b"//"[..])).map(|_| Token::Idiv),
            attempt(bytes(&b"=="[..])).map(|_| Token::Equal),
            attempt(bytes(&b"~="[..])).map(|_| Token::NotEq),
            attempt(bytes(&b"<="[..])).map(|_| Token::LesEq),
            attempt(bytes(&b">="[..])).map(|_| Token::GreEq),
            attempt(bytes(&b"::"[..])).map(|_| Token::DoubColon),
            attempt(bytes(&b".."[..])).map(|_| Token::Concat),
        )),
        choice((
            token(b'+').map(|_| Token::Add),
            token(b'-').map(|_| Token::Sub),
            token(b'*').map(|_| Token::Mul),
            token(b'/').map(|_| Token::Div),
            token(b'%').map(|_| Token::Mod),
            token(b'^').map(|_| Token::Pow),
            token(b'#').map(|_| Token::Len),
            token(b'&').map(|_| Token::BitAnd),
            token(b'~').map(|_| Token::BitXor),
            token(b'|').map(|_| Token::BitOr),
            token(b'<').map(|_| Token::Less),
            token(b'>').map(|_| Token::Greater),
            token(b'=').map(|_| Token::Assign),
            token(b'(').map(|_| Token::ParL),
            token(b')').map(|_| Token::ParR),
            token(b'{').map(|_| Token::CurlyL),
            token(b'}').map(|_| Token::CurlyR),
            token(b'[').map(|_| Token::SqurL),
            token(b']').map(|_| Token::SqurR),
            token(b';').map(|_| Token::SemiColon),
            token(b':').map(|_| Token::Colon),
            token(b',').map(|_| Token::Comma),
            token(b'.').map(|_| Token::Dot),
        )),
    ))
}

fn integer<Input>() -> impl Parser<Input, Output = Token>
where
    Input: Stream<Token = u8>,
{
    from_str(many1::<Vec<_>, _, _>(digit())).map(Token::Integer)
}

fn float<Input>() -> impl Parser<Input, Output = Token>
where
    Input: Stream<Token = u8>,
{
    from_str(recognize::<Vec<_>, _, _>((
        many1::<Vec<_>, _, _>(digit()),
        token(b'.'),
        many1::<Vec<_>, _, _>(digit()),
    )))
    .map(Token::Float)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_elseif() {
        let (tok, rest) = lua_token().parse(&b"elseif"[..]).unwrap();
        assert_eq!(tok, Token::Elseif);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_idiv() {
        let (tok, rest) = lua_token().parse(&b"//"[..]).unwrap();
        assert_eq!(tok, Token::Idiv);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_dots() {
        let (tok, rest) = lua_token().parse(&b"..."[..]).unwrap();
        assert_eq!(tok, Token::Dots);
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_integer() {
        let (tok, rest) = lua_token().parse(&b"123"[..]).unwrap();
        assert_eq!(tok, Token::Integer(123));
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_float() {
        let (tok, rest) = lua_token().parse(&b"123.45"[..]).unwrap();
        assert_eq!(tok, Token::Float(123.45));
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_sentence() {
        let (t1, rest) = lua_token().parse(&br#"print "hello, world!""#[..]).unwrap();
        let (t2, rest) = lua_token().parse(rest).unwrap();
        assert_eq!(t1, Token::Name("print".into()));
        assert_eq!(t2, Token::String("hello, world!".into()));
        assert!(rest.is_empty());
    }
}
