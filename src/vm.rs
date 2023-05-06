use std::collections::HashMap;

use anyhow::bail;

use crate::{bytecode::ByteCode, parse::ParseProto, value::Value};

#[derive(Debug)]
pub struct ExeState {
    globals: HashMap<String, Value>,
    stack: Vec<Value>,
    func_index: usize,
}

impl ExeState {
    pub fn new() -> Self {
        let mut globals = HashMap::new();
        globals.insert("print".into(), Value::Function(lib_print));

        Self {
            globals,
            stack: Vec::new(),
            func_index: 0,
        }
    }

    pub fn execute(&mut self, proto: &ParseProto) -> anyhow::Result<()> {
        for code in &proto.byte_codes {
            match *code {
                ByteCode::GetGlobal(dst, name) => {
                    let name = &proto.constants[name as usize];
                    if let Value::String(key) = name {
                        let v = self.globals.get(key).unwrap_or(&Value::Nil).clone();
                        self.set_stack(dst, v);
                    } else {
                        bail!("invalid global key: {name:?}");
                    }
                }
                ByteCode::LoadConst(dst, c) => {
                    let v = proto.constants[c as usize].clone();
                    self.set_stack(dst, v);
                }
                ByteCode::Call(func, _) => {
                    self.func_index = func as usize;
                    let func = &self.stack[self.func_index];
                    if let Value::Function(f) = func {
                        f(self);
                    } else {
                        bail!("invalid function: {func:?}");
                    }
                }
                ByteCode::LoadNil(dst) => self.set_stack(dst, Value::Nil),
                ByteCode::LoadBool(dst, c) => self.set_stack(dst, Value::Boolean(c)),
                ByteCode::LoadInt(dst, c) => self.set_stack(dst, Value::Integer(c as i64)),
                ByteCode::Move(dst, src) => self.set_stack(dst, self.stack[src as usize].clone()),
            }
        }
        Ok(())
    }

    fn set_stack(&mut self, dst: u8, v: Value) {
        let dst = dst as usize;
        if self.stack.len() <= dst {
            self.stack.resize(dst + 1, Value::Nil);
        }
        self.stack[dst] = v;
    }
}

fn lib_print(state: &mut ExeState) -> i32 {
    println!("{:?}", state.stack[state.func_index + 1]);
    0
}
