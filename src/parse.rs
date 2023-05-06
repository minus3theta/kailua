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
    let mut locals = Vec::new();
    let mut buf = String::new();
    input.read_to_string(&mut buf)?;
    let mut lex = Lex::new(&buf);

    loop {
        match lex.next()? {
            Token::Name(name) => {
                byte_codes.push(load_var(&mut constants, &locals, locals.len(), name));

                match lex.next()? {
                    Token::ParL => {
                        load_exp(
                            &mut byte_codes,
                            &mut constants,
                            &locals,
                            lex.next()?,
                            locals.len() + 1,
                        )?;

                        if lex.next()? != Token::ParR {
                            bail!("expected `)`");
                        }
                    }
                    Token::String(s) => {
                        byte_codes.push(load_const(
                            &mut constants,
                            locals.len() + 1,
                            Value::String(s),
                        ));
                    }
                    _ => bail!("expected string"),
                }
                byte_codes.push(ByteCode::Call(locals.len() as u8, 1));
            }
            Token::Local => {
                let var = if let Token::Name(var) = lex.next()? {
                    var
                } else {
                    bail!("expected variable");
                };

                if lex.next()? != Token::Assign {
                    bail!("expected `=`");
                }

                load_exp(
                    &mut byte_codes,
                    &mut constants,
                    &locals,
                    lex.next()?,
                    locals.len(),
                )?;

                locals.push(var);
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
    constants.iter().position(|v| v == &c).unwrap_or_else(|| {
        constants.push(c);
        constants.len() - 1
    })
}

fn load_const(constants: &mut Vec<Value>, dst: usize, c: Value) -> ByteCode {
    ByteCode::LoadConst(dst as u8, add_const(constants, c) as u8)
}

fn load_exp(
    byte_codes: &mut Vec<ByteCode>,
    constants: &mut Vec<Value>,
    locals: &[String],
    token: Token,
    dst: usize,
) -> anyhow::Result<()> {
    let code = match token {
        Token::Nil => ByteCode::LoadNil(dst as u8),
        Token::True => ByteCode::LoadBool(dst as u8, true),
        Token::False => ByteCode::LoadBool(dst as u8, false),
        Token::Integer(i) => {
            if let Result::Ok(ii) = i16::try_from(i) {
                ByteCode::LoadInt(dst as u8, ii)
            } else {
                load_const(constants, dst, Value::Integer(i))
            }
        }
        Token::Float(f) => load_const(constants, dst, Value::Float(f)),
        Token::String(s) => load_const(constants, dst, Value::String(s)),
        Token::Name(var) => load_var(constants, locals, dst, var),
        _ => bail!("invalid argument"),
    };
    byte_codes.push(code);
    Ok(())
}

fn load_var(constants: &mut Vec<Value>, locals: &[String], dst: usize, name: String) -> ByteCode {
    if let Some(i) = locals.iter().rposition(|v| v == &name) {
        ByteCode::Move(dst as u8, i as u8)
    } else {
        let ic = add_const(constants, Value::String(name));
        ByteCode::GetGlobal(dst as u8, ic as u8)
    }
}
