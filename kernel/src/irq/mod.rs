use crate::object::KernelObjectBase;
use rtl::signal::Signal;

pub struct IrqObject {
    base: KernelObjectBase,
}

crate::kernel_object!(IrqObject, Signal::None.into());

impl IrqObject {

}
