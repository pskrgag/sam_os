use core::sync::atomic::AtomicBool;
use rtl::linker_var;

pub mod test_descr;

pub static TEST_FAIL: AtomicBool = AtomicBool::new(false);

unsafe extern "C" {
    static skerneltests: usize;
    static ekerneltests: usize;
}

#[macro_export]
macro_rules! test_assert {
    ($cond:expr) => {
        if !$cond {
            error!(
                "\nCondition failed: `{}` at {}:{}",
                stringify!($cond),
                file!(),
                line!()
            );
            $crate::tests::TEST_FAIL.store(true, core::sync::atomic::Ordering::Relaxed);
            return;
        }
    };
}

#[macro_export]
macro_rules! test_assert_ne {
    ($e1:expr, $e2:expr) => {
        if $e1 == $e2 {
            error!("\nTest assert failure at {}:{:?} ", file!(), line!());
            error!("Condition failed: `{:?} != {:?}`", $e1, $e2);
            $crate::tests::TEST_FAIL.store(true, core::sync::atomic::Ordering::Relaxed)
            return;
        }
    };
}

#[macro_export]
macro_rules! test_assert_eq {
    ($e1:expr, $e2:expr) => {
        if $e1 != $e2 {
            error!("\nTest assert failure at {}:{:?} ", file!(), line!());
            error!("Condition failed: `{:?} == {:?}`\n", $e1, $e2);
            $crate::tests::TEST_FAIL.store(true, core::sync::atomic::Ordering::Relaxed);
            return;
        }
    };
}

#[cfg(test)]
pub fn test_runner(_tests: &[&dyn Fn()]) {
    use test_descr::*;

    let kernel_tests = unsafe {
        core::slice::from_raw_parts(
            linker_var!(skerneltests) as *const TestDescr,
            (linker_var!(ekerneltests) - linker_var!(skerneltests))
                / core::mem::size_of::<TestDescr>(),
        )
    };

    info!("Running {} tests...\n", kernel_tests.len());

    for test in kernel_tests {
        info!("Running {}::{}\n", test.module, test.name);

        (test.test_fn)();

        info!("\nResult {}::{} ", test.module, test.name);
        if TEST_FAIL.load(core::sync::atomic::Ordering::Relaxed) {
            error!("[FAIL]\n");
        } else {
            info!("[SUCCESS]\n");
        }

        crate::tests::TEST_FAIL.store(false, core::sync::atomic::Ordering::Relaxed)
    }
}
