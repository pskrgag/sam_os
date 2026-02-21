use crate::bindings_BlkDev::BlkDev;
use crate::fs::Filesystem;
use crate::vfs::inode::Inode;
use alloc::sync::Arc;
use dcache::Dcache;
use fs::path::Path;
use libc::handle::Handle;
use rtl::error::ErrorType;
use rtl::locking::spinlock::Spinlock;
use spin::once::Once;

mod dcache;
mod dir;
mod file;
pub mod inode;

pub struct Vfs {
    dcache: Spinlock<Dcache>,
}

static VFS: Once<Arc<Vfs>> = Once::new();

impl Vfs {
    /// Creates new VFS with specified root device
    pub async fn new<S: AsRef<str>>(blk: BlkDev, fs: S) -> Result<Self, ErrorType> {
        let mut s = Self {
            dcache: Spinlock::new(Dcache::new()),
        };

        s.mount(blk, "/", fs, None).await?;
        Ok(s)
    }

    /// Opens a directory
    pub async fn open_dir<'a, S: AsRef<Path<'a>>>(
        &self,
        dir: S,
    ) -> Result<(impl Future<Output = Result<(), ErrorType>>, Handle), ErrorType> {
        let mut path = Some(dir.as_ref());
        let dcache = self.dcache.lock();

        while let Some(p) = path {
            if let Some(dir) = dcache.find::<&str>(p.as_ref().as_ref()) {
                return dir::OpenDirectory::new(dir.clone());
            }

            path = p.parent();
        }

        // We did not find any mount points during lookup, so ask root to lookup this for
        // us
        let root_inode = dcache.find("/").expect("Root should be present").clone();
        let root = root_inode.as_dir().expect("Root must be directory").clone();

        // Drop dcache lock to allow fs to allocate new dcache entries while walking
        drop(dcache);

        root.lookup(dir.as_ref(), &root_inode).await.and_then(|x| {
            if x.is_dir() {
                // Try to insert the inode. It might has been allocated by another user, so lookup
                // might be useless
                Ok(dir::OpenDirectory::new(
                    self.dcache.lock().insert(dir.as_ref(), x.clone()).clone(),
                ))
            } else {
                Err(ErrorType::InvalidArgument)
            }
        })?
    }

    /// Mount device on specified mount point
    pub async fn mount<'a, P: AsRef<Path<'a>>, S: AsRef<str>>(
        &mut self,
        blk: BlkDev,
        to: P,
        name: S,
        parent: Option<Arc<Inode>>,
    ) -> Result<(), ErrorType> {
        let sb = match name.as_ref() {
            "fat32" => crate::fs::fat32::Fat32::try_mount(blk, parent).await?,
            _ => panic!("Unknown FS"),
        };

        self.dcache.lock().insert(to.as_ref().into_owned(), sb);
        Ok(())
    }

    /// Creates a new dcache entry
    pub fn dcache_store<'a, P: AsRef<Path<'a>>>(&self, path: P, inode: Arc<Inode>) -> Arc<Inode> {
        // I'd like to return a reference from here, but fucking BC fucks it up and thinks that
        // lifetime of return value is bound to dcache.lock(). Idk, maybe I messed up spinlock
        // implementation
        self.dcache
            .lock()
            .insert(path.as_ref().into_owned(), inode)
            .clone()
    }

    /// Lookup a dcache entry
    pub fn dcache_lookup<'a, P: AsRef<Path<'a>>>(&'static self, path: P) -> Option<Arc<Inode>> {
        self.dcache.lock().find(path.as_ref()).cloned()
    }
}

pub fn vfs() -> &'static Arc<Vfs> {
    unsafe { VFS.get_unchecked() }
}

pub async fn init<S: AsRef<str>>(blk: BlkDev, fs: S) {
    let vfs = Arc::new(Vfs::new(blk, fs).await.unwrap());

    VFS.call_once(|| vfs);
}
