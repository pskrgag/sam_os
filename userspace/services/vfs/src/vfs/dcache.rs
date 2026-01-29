use super::inode::Inode;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;

pub struct Dcache {
    map: BTreeMap<String, Arc<Inode>>,
}

impl Dcache {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn find<S: AsRef<str>>(&self, path: S) -> Option<&Arc<Inode>> {
        self.map.get(path.as_ref())
    }

    pub fn insert<S: AsRef<str>>(&mut self, path: S, inode: Arc<Inode>) -> &Arc<Inode> {
        self.map.entry(path.as_ref().to_string()).or_insert(inode)
    }
}
