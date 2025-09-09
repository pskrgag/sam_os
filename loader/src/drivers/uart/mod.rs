use crate::log::register_logger;
use core::fmt::Write;
use fdt::{node::FdtNode, Fdt};
use linkme::distributed_slice;

pub mod pl011;

struct UartProbe {
    pub compatible: &'static str,
    pub probe: fn(&FdtNode) -> Option<*mut dyn Write>,
}

#[distributed_slice]
pub static UARTS: [UartProbe];

pub fn probe(fdt: &Fdt) {
    let chosen = fdt.find_node("/chosen").unwrap();
    let stdout = chosen.property("stdout-path").unwrap();
    let stdout_path = unsafe { core::ffi::CStr::from_ptr(stdout.value.as_ptr() as *const _) };
    let node = fdt.find_node(stdout_path.to_str().unwrap()).unwrap();

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
