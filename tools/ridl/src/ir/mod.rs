pub mod interface;
pub mod function;
pub mod argtype;

use std::any::Any;

pub trait IrObject {
    fn as_any(&self) -> &dyn Any;
}
