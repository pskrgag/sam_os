use crate::{
    arch::cpuid::current_cpu,
    arch::PAGE_SIZE,
    linker_var,
    mm::{
        allocators::page_alloc::page_allocator,
        paging::{kernel_page_table::kernel_page_table, page_table::MappingType},
        types::{MemRange, PhysAddr, VirtAddr},
    },
};

use spin::Once;

// TODO: W/A. it should be read from dtb
const NUM_CPUS: usize = 2;

extern "C" {
    static sdatapercpu: usize;
    static edatapercpu: usize;
}

static PER_CPU_BASE: Once<VirtAddr> = Once::new();
static PER_CPU_SIZE: Once<usize> = Once::new();

#[macro_export]
macro_rules! percpu_global {
    ($(#[$attr:meta])* static $N:ident : $T:ty = $e:expr;) => {
        #[link_section = ".percpu.data"]
       $(#[$attr])* static $N : $T = $e;
    };

    ($(#[$attr:meta])* pub static $N:ident : $T:ty = $e:expr;) => {
        #[link_section = ".percpu.data"]
       $(#[$attr])* pub static $N : $T = $e;
    };

    ($(#[$attr:meta])* pub static mut $N:ident : $T:ty = $e:expr;) => {
        #[link_section = ".percpu.data"]
       $(#[$attr])* pub static mut $N : $T = $e;
    };

    ($(#[$attr:meta])* static mut $N:ident : $T:ty = $e:expr;) => {
        #[link_section = ".percpu.data"]
       $(#[$attr])* static mut $N : $T = $e;
    };
}

percpu_global!(
    static TEST: usize = 0x1234;
);

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
            let per_cpu_addr = ((PER_CPU_BASE.get_unchecked().get()
                + current_cpu() * PER_CPU_SIZE.get_unchecked())
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
            let per_cpu_addr = ((PER_CPU_BASE.get_unchecked().get()
                + current_cpu() * PER_CPU_SIZE.get_unchecked())
                + diff) as *mut u8;

            __cast(&$var, per_cpu_addr).as_mut_ref().unwrap()
        }
    }};
}

pub fn init_percpu() -> Option<()> {
    let per_cpu_size = linker_var!(edatapercpu) - linker_var!(sdatapercpu);

    assert!(per_cpu_size % PAGE_SIZE == 0);

    let pages = (per_cpu_size / PAGE_SIZE) * NUM_CPUS;
    let pa: PhysAddr = page_allocator().alloc(pages)?.into();

    PER_CPU_BASE.call_once(|| VirtAddr::from(pa));
    PER_CPU_SIZE.call_once(|| per_cpu_size);

    println!("Per cpu size {}", per_cpu_size);

    kernel_page_table()
        .map(
            None,
            MemRange::new(VirtAddr::from(linker_var!(sdatapercpu)), per_cpu_size),
            MappingType::KernelData,
        )
        .ok()?;

    for i in 0..NUM_CPUS {
        let p = pa + (per_cpu_size * i).into();
        kernel_page_table()
            .map(
                Some(MemRange::new(p, per_cpu_size)),
                MemRange::new(VirtAddr::from(p), per_cpu_size),
                MappingType::KernelData,
            )
            .ok()?;

        #[allow(unused_unsafe)]
        unsafe {
            core::slice::from_raw_parts_mut(VirtAddr::from(p).to_raw_mut::<u8>(), per_cpu_size)
                .copy_from_slice(core::slice::from_raw_parts(
                    linker_var!(sdatapercpu) as *const u8,
                    per_cpu_size,
                ));
        }
    }

    // TODO: unmap?

    Some(())
}

#[no_mangle]
pub extern "C" fn tmp() -> usize {
    *percpu!(TEST)
}
