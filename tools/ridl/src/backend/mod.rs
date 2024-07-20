use crate::ir::interface::Interface;
use crate::ir::IrObject;
use std::io::{Result, Write};
use std::rc::Rc;
use std::sync::atomic::{AtomicU8, Ordering::Relaxed};

pub mod rust;

pub struct PrintContext<'a, B: Write> {
    indent: AtomicU8,
    base: u8,
    stream: &'a mut B,
}

impl<'a, B: Write> PrintContext<'a, B> {
    pub fn new(base: u8, stream: &'a mut B) -> Self {
        Self {
            indent: AtomicU8::new(base),
            base,
            stream,
        }
    }

    pub fn print<S: AsRef<str>>(&mut self, str: S) {
        write!(self.stream, "{}", str.as_ref()).unwrap();
    }

    pub fn println<S: AsRef<str>>(&mut self, str: S) {
        writeln!(self.stream, "{}", str.as_ref()).unwrap();
    }

    pub fn inc_indent(&self) {
        self.indent.fetch_add(self.base, Relaxed);
    }

    pub fn dec_indent(&self) {
        self.indent.fetch_sub(self.base, Relaxed);
    }
}

pub fn compile_transport<B: Write>(
    v: &Vec<Box<dyn IrObject>>,
    out: &mut B,
    lang: &str,
) -> Result<()> {
    match lang {
        "rust" => {
            // let mut b = rust::RustBackend::new(out);
            // b.compile_transport(v)
            todo!()
        }
        &_ => panic!("wtf"),
    }
}

pub fn compile_server<B: Write>(v: &Vec<Box<dyn IrObject>>, out: &mut B, lang: &str) -> Result<()> {
    for i in v {
        if let Some(interface) = i.as_any().downcast_ref::<Interface>() {
            match lang {
                "rust" => {
                    let mut b = rust::RustBackend::new(out);
                    b.compile_server(interface)?;
                }
                &_ => panic!("wtf"),
            }
        } else {
            panic!("Wrong ir");
        }
    }

    Ok(())
}
