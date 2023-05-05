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
                constants.push(Value::String(name));
                byte_codes.push(ByteCode::GetGlobal(0, (constants.len() - 1) as u8));

                if let Token::String(s) = lex.next()? {
                    constants.push(Value::String(s));
                    byte_codes.push(ByteCode::LoadConst(1, (constants.len() - 1) as u8));
                    byte_codes.push(ByteCode::Call(0, 1));
                }
            }
            Token::Eos => break,
            t => bail!("unexpected token: {t:?}"),
        }
    }

    dbg!(&constants);
    dbg!(&byte_codes);

    Ok(ParseProto {
        constants,
        byte_codes,
    })
}
