use crate::ast::{
    argtype::Type,
    function::{Argument, Function},
    interface::Interface,
    module::Module,
};
use std::io::Write;

#[derive(Clone)]
struct Struct {
    name: String,
    data: Vec<(String, Type)>,
}

struct InterfaceCompiler<'a, W: Write> {
    interface: &'a Interface,
    buf: &'a mut W,
    tx: Vec<Struct>,
    rx: Vec<Struct>,
}

impl<'a, W: Write> InterfaceCompiler<'a, W> {
    fn includes(&mut self) {
        writeln!(self.buf, "use rtl::handle::Handle;").unwrap();
        writeln!(self.buf, "use rtl::ipc::IpcMessage;").unwrap();
        writeln!(self.buf, "use rtl::error::ErrorType;").unwrap();
        writeln!(self.buf, "use serde::{{Deserialize, Serialize}};").unwrap();
        writeln!(self.buf, "use postcard::{{to_allocvec, from_bytes}};").unwrap();
        writeln!(self.buf, "use libc::port::Port;").unwrap();
        writeln!(self.buf).unwrap();
    }

    fn make_struct(&mut self) {
        writeln!(
            self.buf,
            "struct {} {{\n    handle: Handle,\n}}\n",
            self.interface.name()
        )
        .unwrap();
    }

    fn begin_impl(&mut self) {
        write!(self.buf, "impl {} {{", self.interface.name()).unwrap();
        writeln!(
            self.buf,
            r#"
    pub fn new(handle: Handle) -> Self {{
        Self {{ handle }}
    }}
"#,
        )
        .unwrap();
    }

    fn end_impl(&mut self) {
        writeln!(self.buf, "}}\n").unwrap();
    }

    fn compile_function(&mut self, f: &Function) {
        let mut rx = vec![];
        let mut tx = vec![];

        write!(self.buf, "    pub fn {}(&self", f.name()).unwrap();

        for arg in f.args() {
            match arg {
                Argument::In(tp, name) => {
                    tx.push((name.clone(), tp.clone()));
                    write!(self.buf, ", {}: {}", name, tp).unwrap();
                }
                Argument::Out(tp, name) => rx.push((name.clone(), tp.clone())),
            }
        }

        {
            writeln!(
                self.buf,
                r#") -> Result<{}Output, ErrorType> {{
        let mut message = IpcMessage::new();
        let data = {}Input {{ {} }};
        let data_vec = to_allocvec(&data).unwrap();
        let mut receive_buffer = [0u8; core::mem::size_of::<{}Output>()];

        message.set_out_arena(data_vec.as_slice());
        message.set_in_arena(receive_buffer.as_mut_slice());

        Port::new(self.handle).call(&mut message)?;

        let res = from_bytes(message.in_data.unwrap()).unwrap();
        Ok(res)"#,
                f.name(),
                f.name(),
                tx.iter().map(|x| x.0.clone()).collect::<Vec<_>>().join(","),
                f.name(),
            )
            .unwrap();

            writeln!(self.buf, "    }}").unwrap();
        }

        self.tx.push(Struct {
            data: tx,
            name: f.name().to_string(),
        });
        self.rx.push(Struct {
            data: rx,
            name: f.name().to_string(),
        });
    }

    fn produce_compound_enum(&mut self, s: &Struct, suffix: &str) {
        writeln!(
            self.buf,
            "#[derive(Serialize, Deserialize, Debug)]\nstruct {}{} {{",
            s.name, suffix
        )
        .unwrap();

        for data in &s.data {
            writeln!(self.buf, "    {}: {}", data.0, data.1).unwrap();
        }

        writeln!(self.buf, "}}").unwrap();
    }

    fn produce_enum(&mut self, name: &str, suffix: &str, data: Vec<Struct>) {
        for tx in &data {
            self.produce_compound_enum(tx, suffix);
        }

        writeln!(
            self.buf,
            "#[derive(Serialize, Deserialize, Debug)]\nenum {name} {{"
        )
        .unwrap();
        for tx in data {
            writeln!(self.buf, "    {}({}{})", tx.name, tx.name, suffix).unwrap();
        }

        writeln!(self.buf, "}}").unwrap();
    }

    fn produce_enums(&mut self) {
        self.produce_enum("Tx", "Input", self.tx.clone());
        self.produce_enum("Rx", "Output", self.rx.clone());
    }

    pub fn compile(mut self) {
        self.includes();
        self.make_struct();

        self.begin_impl();
        for func in self.interface.functions() {
            self.compile_function(func);
        }
        self.end_impl();

        self.produce_enums();
    }
}

pub fn compile_client<W: Write>(ir: Module, buf: &mut W) {
    for interface in ir.interfaces() {
        InterfaceCompiler {
            interface,
            buf,
            tx: vec![],
            rx: vec![],
        }
        .compile()
    }
}
