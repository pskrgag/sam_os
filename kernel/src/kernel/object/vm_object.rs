use object_lib::object;


#[derive(object)]
pub struct VmObject {
    start: usize,
    end: usize,
}
