use crate::errors::AppError;
use crate::proc::{self, mem::ProcessHandle};

use mlua::{
    Lua, MetaMethod, UserData, UserDataMethods, Value,
    prelude::{LuaResult, LuaTable},
};
use std::sync::Arc;

enum MemType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
}

struct TypedView {
    m_type: MemType,
    handle: Arc<ProcessHandle>,
}

impl UserData for TypedView {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::Index, |_, this, addr: usize| {
            let addr_u64 = addr as u64;
            let read_err = || mlua::Error::RuntimeError(format!("内存读取失败: {addr:#X}"));

            match this.m_type {
                MemType::U8 => Ok(Value::Integer(
                    this.handle.read_val::<u8>(addr_u64).ok_or_else(read_err)? as i64,
                )),
                MemType::I8 => Ok(Value::Integer(
                    this.handle.read_val::<i8>(addr_u64).ok_or_else(read_err)? as i64,
                )),
                MemType::U16 => Ok(Value::Integer(
                    this.handle.read_val::<u16>(addr_u64).ok_or_else(read_err)? as i64,
                )),
                MemType::I16 => Ok(Value::Integer(
                    this.handle.read_val::<i16>(addr_u64).ok_or_else(read_err)? as i64,
                )),
                MemType::U32 => Ok(Value::Integer(
                    this.handle.read_val::<u32>(addr_u64).ok_or_else(read_err)? as i64,
                )),
                MemType::I32 => Ok(Value::Integer(
                    this.handle.read_val::<i32>(addr_u64).ok_or_else(read_err)? as i64,
                )),
                MemType::U64 => Ok(Value::Integer(
                    this.handle.read_val::<u64>(addr_u64).ok_or_else(read_err)? as i64,
                )),
                MemType::I64 => Ok(Value::Integer(
                    this.handle.read_val::<i64>(addr_u64).ok_or_else(read_err)?,
                )),
                MemType::F32 => Ok(Value::Number(
                    this.handle.read_val::<f32>(addr_u64).ok_or_else(read_err)? as f64,
                )),
                MemType::F64 => Ok(Value::Number(
                    this.handle.read_val::<f64>(addr_u64).ok_or_else(read_err)?,
                )),
            }
        });

        methods.add_meta_method(
            MetaMethod::NewIndex,
            |_, this, (addr, val): (usize, Value)| {
                let addr_u64 = addr as u64;
                let write_err = || mlua::Error::RuntimeError(format!("内存写入失败: {addr:#X}"));

                match this.m_type {
                    MemType::U8
                    | MemType::I8
                    | MemType::U16
                    | MemType::I16
                    | MemType::U32
                    | MemType::I32
                    | MemType::U64
                    | MemType::I64 => {
                        let num = val
                            .as_i64()
                            .ok_or_else(|| mlua::Error::RuntimeError("赋值必须是整数".into()))?;

                        let success = match this.m_type {
                            MemType::U8 => this.handle.write_val(addr_u64, &(num as u8)),
                            MemType::I8 => this.handle.write_val(addr_u64, &(num as i8)),
                            MemType::U16 => this.handle.write_val(addr_u64, &(num as u16)),
                            MemType::I16 => this.handle.write_val(addr_u64, &(num as i16)),
                            MemType::U32 => this.handle.write_val(addr_u64, &(num as u32)),
                            MemType::I32 => this.handle.write_val(addr_u64, &(num as i32)),
                            MemType::U64 => this.handle.write_val(addr_u64, &(num as u64)),
                            MemType::I64 => this.handle.write_val(addr_u64, &num),
                            _ => unreachable!(),
                        };

                        if !success {
                            return Err(write_err());
                        }
                    }
                    MemType::F32 | MemType::F64 => {
                        let num = val
                            .as_f64()
                            .ok_or_else(|| mlua::Error::RuntimeError("赋值必须是数字".into()))?;

                        let success = match this.m_type {
                            MemType::F32 => this.handle.write_val(addr_u64, &(num as f32)),
                            MemType::F64 => this.handle.write_val(addr_u64, &num),
                            _ => unreachable!(),
                        };

                        if !success {
                            return Err(write_err());
                        }
                    }
                }

                Ok(())
            },
        );
    }
}

pub fn load_mem_view_module(lua: &Lua) -> LuaResult<LuaTable> {
    let view_table = lua.create_table()?;

    let handle = proc::get_handle().ok_or(AppError::ProcessHandleNotFound)?;

    view_table.set(
        "u8",
        TypedView {
            m_type: MemType::U8,
            handle: handle.clone(),
        },
    )?;
    view_table.set(
        "i8",
        TypedView {
            m_type: MemType::I8,
            handle: handle.clone(),
        },
    )?;
    view_table.set(
        "u16",
        TypedView {
            m_type: MemType::U16,
            handle: handle.clone(),
        },
    )?;
    view_table.set(
        "i16",
        TypedView {
            m_type: MemType::I16,
            handle: handle.clone(),
        },
    )?;
    view_table.set(
        "u32",
        TypedView {
            m_type: MemType::U32,
            handle: handle.clone(),
        },
    )?;
    view_table.set(
        "i32",
        TypedView {
            m_type: MemType::I32,
            handle: handle.clone(),
        },
    )?;
    view_table.set(
        "u64",
        TypedView {
            m_type: MemType::U64,
            handle: handle.clone(),
        },
    )?;
    view_table.set(
        "i64",
        TypedView {
            m_type: MemType::I64,
            handle: handle.clone(),
        },
    )?;
    view_table.set(
        "f32",
        TypedView {
            m_type: MemType::F32,
            handle: handle.clone(),
        },
    )?;
    view_table.set(
        "f64",
        TypedView {
            m_type: MemType::F64,
            handle: handle.clone(),
        },
    )?;

    view_table.set(
        "readPointerChain",
        lua.create_function(move |_, (addr, offsets): (usize, Vec<usize>)| {
            if offsets.is_empty() {
                return Err(mlua::Error::RuntimeError("偏移量列表不能为空".into()));
            }
            handle
                .clone()
                .read_pointer_chain(addr as u64, &offsets)
                .map(|v| Value::Integer(v as i64))
                .ok_or_else(|| mlua::Error::RuntimeError("指针链解析失败".into()))
        })?,
    )?;

    Ok(view_table)
}
