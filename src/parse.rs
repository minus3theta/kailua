use std::{fs::File, io::Read};

use anyhow::{bail, Ok};

use crate::{
    bytecode::ByteCode,
    lex::{Lex, Token},
    value::Value,
};

#[derive(Debug)]
pub struct ParseProto {
    pub constants: Vec<Value>,
    pub byte_codes: Vec<ByteCode>,
}

pub fn load(mut input: File) -> anyhow::Result<ParseProto> {
    let mut constants = Vec::new();
    let mut byte_codes = Vec::new();
    let mut buf = String::new();
    input.read_to_string(&mut buf)?;
    let mut lex = Lex::new(&buf);

    loop {
        match lex.next()? {
            Token::Name(name) => {
                let ic = add_const(&mut constants, Value::String(name));
                byte_codes.push(ByteCode::GetGlobal(0, ic as u8));

                match lex.next()? {
                    Token::ParL => {
                        let code = match lex.next()? {
                            Token::Nil => ByteCode::LoadNil(1),
                            Token::True => ByteCode::LoadBool(1, true),
                            Token::False => ByteCode::LoadBool(1, false),
                            Token::Integer(i) => {
                                if let Result::Ok(ii) = i16::try_from(i) {
                                    ByteCode::LoadInt(1, ii)
                                } else {
                                    load_const(&mut constants, 1, Value::Integer(i))
                                }
                            }
                            Token::Float(f) => load_const(&mut constants, 1, Value::Float(f)),
                            Token::String(s) => load_const(&mut constants, 1, Value::String(s)),
                            _ => bail!("invalid argument"),
                        };
                        byte_codes.push(code);

                        if lex.next()? != Token::ParR {
                            bail!("expected `)`");
                        }
                    }
                    Token::String(s) => {
                        byte_codes.push(load_const(&mut constants, 1, Value::String(s)));
                    }
                    _ => bail!("expected string"),
                }
                byte_codes.push(ByteCode::Call(0, 1));
            }
            Token::Eos => break,
            t => bail!("unexpected token: {t:?}"),
        }
    }

    dbg!(&constants);
    eprintln!("byte_codes:");
    for code in &byte_codes {
        eprintln!("    {code:?}");
    }

    Ok(ParseProto {
        constants,
        byte_codes,
    })
}

fn add_const(constants: &mut Vec<Value>, c: Value) -> usize {
    constants.push(c);
    constants.len() - 1
}

fn load_const(constants: &mut Vec<Value>, dst: usize, c: Value) -> ByteCode {
    ByteCode::LoadConst(dst as u8, add_const(constants, c) as u8)
}
