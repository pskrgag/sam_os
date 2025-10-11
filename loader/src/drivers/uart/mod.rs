use crate::log::register_logger;
use core::fmt::Write;
use fdt::{Fdt, node::FdtNode};
use linkme::distributed_slice;
use loader_protocol::LoaderArg;

pub mod pl011;

struct UartProbe {
    pub compatible: &'static str,
    pub probe: fn(&FdtNode) -> Option<*mut dyn Write>,
    pub map: fn(&FdtNode, &mut LoaderArg),
}

#[distributed_slice]
pub static UARTS: [UartProbe];

fn get_stdout<'a>(fdt: &'a Fdt) -> FdtNode<'a, 'a> {
    let chosen = fdt.find_node("/chosen").unwrap();
    let stdout = chosen.property("stdout-path").unwrap();
    let stdout_path = unsafe { core::ffi::CStr::from_ptr(stdout.value.as_ptr() as *const _) };

    fdt.find_node(stdout_path.to_str().unwrap()).unwrap()
}

pub fn probe(fdt: &Fdt) {
    let node = get_stdout(fdt);

    for uart_drv in UARTS {
        if node
            .compatible()
            .unwrap()
            .all()
            .any(|x| x == uart_drv.compatible)
        {
            if let Some(uart) = (uart_drv.probe)(&node) {
                register_logger(uart);
                println!("Using {} as stdout", uart_drv.compatible);
                break;
            }
        }
    }
}

pub fn map(fdt: &Fdt, arg: &mut LoaderArg) {
    let node = get_stdout(fdt);

    for uart_drv in UARTS {
        if node
            .compatible()
            .unwrap()
            .all()
            .any(|x| x == uart_drv.compatible)
        {
            (uart_drv.map)(&node, arg);
        }
    }
}
