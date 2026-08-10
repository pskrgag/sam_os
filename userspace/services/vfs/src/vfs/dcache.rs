use super::inode::Inode;
use crate::bindings_Vfs::{DirEntry, DirEntryKind};
use alloc::collections::btree_map::Entry;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use fs::path::Path;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;

pub enum CreateType {
    File,
    Directory,
}

#[derive(Default)]
pub struct Cache {
    children: BTreeMap<String, Arc<Dentry>>,
    uptodate: bool,
}

/// Cached file system entry
pub struct Dentry {
    parent: Option<Weak<Dentry>>,
    inode: Arc<Inode>,
    cache: Spinlock<Cache>,
}

impl Cache {
    fn to_direntry(&self) -> Vec<DirEntry> {
        let mut res = Vec::new();

        for (name, dentry) in self.children.iter() {
            res.push(DirEntry {
                name: name.as_str().try_into().unwrap(),
                flags: if dentry.is_dir() {
                    DirEntryKind::Directory
                } else {
                    DirEntryKind::File
                },
            });
        }

        res
    }

    fn insert_child(&mut self, parent: &Arc<Dentry>, name: &str, inode: Arc<Inode>) -> Arc<Dentry> {
        match self.children.entry(name.to_string()) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let child = Dentry::new_child(parent, inode);

                entry.insert(child.clone());
                child
            }
        }
    }
}

impl Dentry {
    /// Creates root dentry
    pub fn new_root(inode: Arc<Inode>) -> Arc<Self> {
        Arc::new(Self {
            parent: None,
            inode,
            cache: Spinlock::new(Cache::default()),
        })
    }

    /// Returns dentry parent
    pub fn parent(&self) -> Option<Arc<Dentry>> {
        self.parent.as_ref().and_then(Weak::upgrade)
    }

    /// Returns dentry inode
    pub fn inode(&self) -> &Arc<Inode> {
        &self.inode
    }

    /// Checks if dentry points to directory
    pub fn is_dir(&self) -> bool {
        self.inode.is_dir()
    }

    /// Looks up entry specified by path
    async fn lookup_components(
        self: &Arc<Self>,
        components: &[&str],
    ) -> Result<Arc<Dentry>, ErrorType> {
        let mut current = self.clone();

        for comp in components {
            match *comp {
                ".." => {
                    if let Some(parent) = current.parent() {
                        current = parent;
                    } else {
                        return Err(ErrorType::InvalidArgument);
                    }
                }
                comp => {
                    if let Some(child) = current.lookup_child(comp) {
                        current = child;
                    } else if let Some(dir) = current.inode.as_dir() {
                        let file = dir.lookup(comp).await?;

                        current = Dentry::insert_child(&current, comp, file);
                    } else {
                        return Err(ErrorType::InvalidArgument);
                    }
                }
            };
        }

        Ok(current)
    }

    /// Looks up entry specified by path
    pub async fn lookup_or_create<'a, P: Into<Path<'a>>>(
        self: &Arc<Self>,
        path: P,
        kind: Option<CreateType>,
    ) -> Result<Arc<Dentry>, ErrorType> {
        let path = path.into();
        let components = path.components().collect::<Vec<_>>();
        let name = components.last().unwrap();

        let dir = self
            .lookup_components(&components[..components.len() - 1])
            .await?;

        let Some(dir_ops) = dir.inode.as_dir() else {
            return Err(ErrorType::InvalidArgument);
        };

        let file = match dir.lookup_components(&[name]).await {
            Err(ErrorType::NotFound) => {
                // Dcache + disk lookup failed. We need to physically create smth
                let inode = match kind {
                    Some(CreateType::File) => dir_ops.create_file(name).await?,
                    Some(CreateType::Directory) => dir_ops.create_directory(name).await?,
                    _ => return Err(ErrorType::NotFound),
                };

                Dentry::insert_child(&dir, name, inode.clone())
            }
            e => e?,
        };

        Ok(file)
    }

    /// Lists directory content
    pub async fn list(self: &Arc<Self>) -> Result<Vec<DirEntry>, ErrorType> {
        let Some(dir) = self.inode.as_dir() else {
            return Err(ErrorType::InvalidArgument);
        };

        let mut cache = self.cache.lock();

        let mut res = if cache.uptodate {
            cache.to_direntry()
        } else {
            let disk_content = {
                drop(cache);

                let disk_content = dir.list().await?;
                cache = self.cache.lock();
                disk_content
            };

            // TODO: actually it would be great to make list() return Inode... Need to refactor
            // stuff here
            for i in disk_content {
                if !cache.children.contains_key(i.name.as_str()) {
                    let inode = dir.lookup(&i.name).await?;

                    cache.insert_child(self, &i.name, inode);
                }
            }

            cache.uptodate = true;
            cache.to_direntry()
        };

        if self.parent().is_some() {
            res.push(DirEntry {
                name: "..".try_into().unwrap(),
                flags: DirEntryKind::Directory,
            });
        }

        Ok(res)
    }

    fn insert_child(parent: &Arc<Self>, name: &str, inode: Arc<Inode>) -> Arc<Dentry> {
        let mut cache = parent.cache.lock();

        cache.insert_child(parent, name, inode)
    }

    fn new_child(parent: &Arc<Self>, inode: Arc<Inode>) -> Arc<Self> {
        Arc::new(Self {
            parent: Some(Arc::downgrade(parent)),
            inode,
            cache: Spinlock::new(Cache::default()),
        })
    }

    fn lookup_child(&self, name: &str) -> Option<Arc<Dentry>> {
        self.cache.lock().children.get(name).cloned()
    }
}
