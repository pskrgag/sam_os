pub mod argtype;
pub mod function;
pub mod interface;

use std::any::Any;

pub trait IrObject {
    fn as_any(&self) -> &dyn Any;
}
