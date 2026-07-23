use super::inode::Inode;
use alloc::collections::btree_map::Entry;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use fs::path::{Components, Path};
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;

/// Cached file system entry
pub struct Dentry {
    name: String,
    parent: Option<Weak<Dentry>>,
    inode: Arc<Inode>,
    children: Spinlock<BTreeMap<String, Arc<Dentry>>>,
}

impl Dentry {
    /// Creates root dentry
    pub fn new_root(inode: Arc<Inode>) -> Arc<Self> {
        Arc::new(Self {
            name: String::from("/"),
            parent: None,
            inode,
            children: Spinlock::new(BTreeMap::new()),
        })
    }

    /// Returns dentry name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns dentry parent
    pub fn parent(&self) -> Option<Arc<Dentry>> {
        self.parent.as_ref().and_then(Weak::upgrade)
    }

    /// Returns dentry inode
    pub fn inode(&self) -> &Arc<Inode> {
        &self.inode
    }

    /// Looks up entry specified by path
    async fn lookup_components(
        self: &Arc<Self>,
        components: &[&str],
    ) -> Result<Arc<Dentry>, ErrorType> {
        let mut current = self.clone();

        for comp in components {
            if let Some(child) = current.lookup_child(comp) {
                current = child;
            } else if let Some(dir) = current.inode.as_dir() {
                let file = dir.lookup(comp).await?;

                current = Dentry::insert_child(&current, comp, file);
            } else {
                return Err(ErrorType::InvalidArgument);
            }
        }

        Ok(current)
    }

    /// Looks up entry specified by path
    pub async fn lookup<'a, P: Into<Path<'a>>>(
        self: &Arc<Self>,
        path: P,
    ) -> Result<Arc<Dentry>, ErrorType> {
        let path = path.into();
        let components = path.components().collect::<Vec<_>>();

        self.lookup_components(&components).await
    }

    pub async fn create_dir<'a, P: Into<Path<'a>>>(
        self: &Arc<Self>,
        path: P,
    ) -> Result<Arc<Dentry>, ErrorType> {
        let path = path.into();
        let components = path.components().collect::<Vec<_>>();

        let parent = self
            .lookup_components(&components[..components.len() - 1])
            .await?;

        let res = if let Some(dir) = parent.inode().as_dir() {
            todo!()
        } else {
            Err(ErrorType::InvalidArgument)
        }?;

        Dentry::insert_child(res, components.last().unwrap(), res.inode.clone());
        Ok(res.clone())
    }

    fn insert_child(parent: &Arc<Self>, name: &str, inode: Arc<Inode>) -> Arc<Dentry> {
        let mut children = parent.children.lock();

        match children.entry(name.to_string()) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let child = Self::new_child(parent, entry.key().clone(), inode);

                entry.insert(child.clone());
                child
            }
        }
    }

    fn new_child(parent: &Arc<Self>, name: String, inode: Arc<Inode>) -> Arc<Self> {
        Arc::new(Self {
            name,
            parent: Some(Arc::downgrade(parent)),
            inode,
            children: Spinlock::new(BTreeMap::new()),
        })
    }

    fn remove_child(&self, name: &str) -> Option<Arc<Dentry>> {
        self.children.lock().remove(name)
    }

    fn lookup_child(&self, name: &str) -> Option<Arc<Dentry>> {
        self.children.lock().get(name).cloned()
    }
}
