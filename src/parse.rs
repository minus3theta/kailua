use std::{fs::File, io::Read};

use anyhow::{bail, Ok};

use crate::{
    bytecode::ByteCode,
    lex::{Lex, Token},
    value::Value,
};

#[derive(Debug)]
struct ParseProtoBuilder<'a> {
    constants: Vec<Value>,
    byte_codes: Vec<ByteCode>,
    locals: Vec<String>,
    lex: Lex<'a>,
}

impl<'a> ParseProtoBuilder<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            constants: Default::default(),
            byte_codes: Default::default(),
            locals: Default::default(),
            lex: Lex::new(input),
        }
    }

    fn load(mut self) -> anyhow::Result<ParseProto> {
        loop {
            match self.lex.next()? {
                Token::Name(name) => {
                    let code = self.load_var(self.locals.len(), name);
                    self.byte_codes.push(code);

                    match self.lex.next()? {
                        Token::ParL => {
                            self.load_exp(self.locals.len() + 1)?;

                            if self.lex.next()? != Token::ParR {
                                bail!("expected `)`");
                            }
                        }
                        Token::String(s) => {
                            let code = self.load_const(self.locals.len() + 1, Value::String(s));
                            self.byte_codes.push(code);
                        }
                        _ => bail!("expected string"),
                    }
                    self.byte_codes
                        .push(ByteCode::Call(self.locals.len() as u8, 1));
                }
                Token::Local => {
                    let var = if let Token::Name(var) = self.lex.next()? {
                        var
                    } else {
                        bail!("expected variable");
                    };

                    if self.lex.next()? != Token::Assign {
                        bail!("expected `=`");
                    }

                    self.load_exp(self.locals.len())?;

                    self.locals.push(var);
                }
                Token::Eos => break,
                t => bail!("unexpected token: {t:?}"),
            }
        }

        dbg!(&self.constants);
        eprintln!("byte_codes:");
        for code in &self.byte_codes {
            eprintln!("    {code:?}");
        }

        Ok(ParseProto {
            constants: self.constants,
            byte_codes: self.byte_codes,
        })
    }

    fn add_const(&mut self, c: Value) -> usize {
        self.constants
            .iter()
            .position(|v| v == &c)
            .unwrap_or_else(|| {
                self.constants.push(c);
                self.constants.len() - 1
            })
    }

    fn load_const(&mut self, dst: usize, c: Value) -> ByteCode {
        ByteCode::LoadConst(dst as u8, self.add_const(c) as u8)
    }

    fn load_exp(&mut self, dst: usize) -> anyhow::Result<()> {
        let code = match self.lex.next()? {
            Token::Nil => ByteCode::LoadNil(dst as u8),
            Token::True => ByteCode::LoadBool(dst as u8, true),
            Token::False => ByteCode::LoadBool(dst as u8, false),
            Token::Integer(i) => {
                if let Result::Ok(ii) = i16::try_from(i) {
                    ByteCode::LoadInt(dst as u8, ii)
                } else {
                    self.load_const(dst, Value::Integer(i))
                }
            }
            Token::Float(f) => self.load_const(dst, Value::Float(f)),
            Token::String(s) => self.load_const(dst, Value::String(s)),
            Token::Name(var) => self.load_var(dst, var),
            _ => bail!("invalid argument"),
        };
        self.byte_codes.push(code);
        Ok(())
    }

    fn load_var(&mut self, dst: usize, name: String) -> ByteCode {
        if let Some(i) = self.locals.iter().rposition(|v| v == &name) {
            ByteCode::Move(dst as u8, i as u8)
        } else {
            let ic = self.add_const(Value::String(name));
            ByteCode::GetGlobal(dst as u8, ic as u8)
        }
    }
}

#[derive(Debug)]
pub struct ParseProto {
    pub constants: Vec<Value>,
    pub byte_codes: Vec<ByteCode>,
}

impl ParseProto {
    pub fn load(mut input: File) -> anyhow::Result<Self> {
        let mut buf = String::new();
        input.read_to_string(&mut buf)?;
        let builder = ParseProtoBuilder::new(&buf);

        builder.load()
    }
}
