use crate::{
    arch::cpuid::current_cpu,
    mm::{allocators::page_alloc::page_allocator, layout::vmm_range},
};
use core::sync::atomic::{AtomicBool, Ordering};

use hal::address::*;
use hal::arch::PAGE_SIZE;
use rtl::linker_var;
use spin::Once;

// TODO: W/A. it should be read from dtb
const NUM_CPUS: usize = 2;

unsafe extern "C" {
    static sdatapercpu: usize;
    static edatapercpu: usize;
}

static PER_CPU_BASE: Once<LinearAddr> = Once::new();
static PER_CPU_SIZE: Once<usize> = Once::new();

// Fake struct to disallow direct usage
pub struct PerCpu<T> {
    data: T,
}

unsafe impl<T> Send for PerCpu<T> {}
unsafe impl<T> Sync for PerCpu<T> {}

#[macro_export]
macro_rules! percpu_global {
    ($(#[$attr:meta])* static $N:ident : $T:ty = $e:expr;) => {
        #[unsafe(link_section = ".percpu.data")]
        $(#[$attr])* static $N : $crate::kernel::percpu::PerCpu<$T> = $crate::kernel::percpu::PerCpu::new($e);
    };

    ($(#[$attr:meta])* pub static $N:ident : $T:ty = $e:expr;) => {
        #[unsafe(link_section = ".percpu.data")]
        $(#[$attr])* pub static $N : $crate::kernel::percpu::PerCpu<$T> = $crate::kernel::percpu::PerCpu::new($e);
    };

    ($(#[$attr:meta])* pub static mut $N:ident : $T:ty = $e:expr;) => {
        #[unsafe(link_section = ".percpu.data")]
        $(#[$attr])* pub static mut $N : $crate::kernel::percpu::PerCpu<$T> = $crate::kernel::percpu::PerCpu::new($e);
    };

    ($(#[$attr:meta])* static mut $N:ident : $T:ty = $e:expr;) => {
        #[unsafe(link_section = ".percpu.data")]
        $(#[$attr])* static mut $N : $crate::kernel::percpu::PerCpu<$T> = $crate::kernel::percpu::PerCpu::new($e);
    };
}

fn __cast<T>(_: &T, ptr: *const u8) -> *const T {
    ptr as *const T
}

fn __cast_mut<T>(_: &T, ptr: *mut u8) -> *mut T {
    ptr as *mut T
}

macro_rules! percpu {
    ($var:expr) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let addr = &$var as *const _ as usize;
            let diff = addr - linker_var!(sdatapercpu);
            let per_cpu_addr = ((PER_CPU_BASE.get_unchecked().bits()
                + current_cpu() * PER_CPU_SIZE.get_unchecked())
                + diff) as *const u8;

            __cast(&$var, per_cpu_addr).as_ref().unwrap()
        }
    }};
}

macro_rules! percpu_n {
    ($var:expr, $n:expr) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let addr = &$var as *const _ as usize;
            let diff = addr - linker_var!(sdatapercpu);
            let per_cpu_addr = ((PER_CPU_BASE.get_unchecked().bits()
                + $n * PER_CPU_SIZE.get_unchecked())
                + diff) as *const u8;

            __cast(&$var, per_cpu_addr).as_ref().unwrap()
        }
    }};
}

macro_rules! percpu_mut {
    ($var:expr) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let addr = &$var as *const _ as usize;
            let diff = addr - linker_var!(sdatapercpu);
            let per_cpu_addr = ((PER_CPU_BASE.get_unchecked().bits()
                + current_cpu() * PER_CPU_SIZE.get_unchecked())
                + diff) as *mut u8;

            __cast_mut(&$var, per_cpu_addr).as_mut().unwrap()
        }
    }};
}

impl<T> PerCpu<T> {
    pub const fn new(data: T) -> Self {
        Self { data }
    }

    pub fn per_cpu_var_get(&self) -> &'static T {
        // assert!(crate::arch::irq::is_disabled());
        percpu!(self.data)
    }

    pub fn per_cpu_var_get_mut(&self) -> &'static mut T {
        percpu_mut!(self.data)
    }

    // SAFETY: caller should know what he is doing, percpu vars are expected to be touched
    // only by owner cpu. IOW caller takes care of synchronization and possible side-effects
    #[allow(dead_code)]
    pub unsafe fn for_each_cpu<F: Fn(&T)>(&self, visiter: F) {
        for i in 0..NUM_CPUS {
            visiter(percpu_n!(self.data, i));
        }
    }

    #[allow(dead_code)]
    pub unsafe fn cpu(&self, cpu: usize) -> &'static T {
        percpu_n!(self.data, cpu)
    }
}

static READY: AtomicBool = AtomicBool::new(false);

pub fn percpu_ready() -> bool {
    READY.load(Ordering::Relaxed)
}

pub fn init_percpu() -> Option<()> {
    let per_cpu_size = linker_var!(edatapercpu) - linker_var!(sdatapercpu);

    assert!(per_cpu_size % PAGE_SIZE == 0);

    let pages = (per_cpu_size / PAGE_SIZE) * NUM_CPUS;
    let pa: PhysAddr = page_allocator()
        .alloc(pages)
        .expect("Failed to allocate memory for per-cpu");

    PER_CPU_BASE.call_once(|| LinearAddr::from(pa));
    PER_CPU_SIZE.call_once(|| per_cpu_size);

    println!("Per cpu size {}", per_cpu_size);
    let range = vmm_range(loader_protocol::VmmLayoutKind::PerCpu);
    debug_assert!(range.size() >= per_cpu_size);

    for i in 0..NUM_CPUS {
        let p = pa + (per_cpu_size * i).into();

        unsafe {
            let linear = LinearAddr::from(p);
            let va: VirtAddr = linear.into();

            core::slice::from_raw_parts_mut(va.to_raw_mut::<u8>(), per_cpu_size).copy_from_slice(
                core::slice::from_raw_parts(linker_var!(sdatapercpu) as *const u8, per_cpu_size),
            );
        }
    }

    READY.store(true, Ordering::Relaxed);
    Some(())
}
