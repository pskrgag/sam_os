use rtl::locking::fake_lock::FakeLock;

pub mod test_descr;

pub static TEST_FAIL: FakeLock<bool> = FakeLock::new(false);

extern "C" {
    static skerneltests: usize;
    static ekerneltests: usize;
}

#[macro_export]
macro_rules! test_assert {
    ($cond:expr) => {
        if !$cond {
            print!("\nCondition failed: `{}`", stringify!($cond));
            *crate::tests::TEST_FAIL.get() = true;
            return;
        }
    };
}

#[macro_export]
macro_rules! test_assert_ne {
    ($e1:expr, $e2:expr) => {
        if $e1 == $e2 {
            print!("\nTest assert failure at {}:{:?} ", file!(), line!());
            print!("Condition failed: `{:?} != {:?}`", $e1, $e2);
            *crate::tests::TEST_FAIL.get() = true;
            return;
        }
    };
}

#[macro_export]
macro_rules! test_assert_eq {
    ($e1:expr, $e2:expr) => {
        if $e1 != $e2 {
            print!("\nTest assert failure at {}:{:?} ", file!(), line!());
            print!("Condition failed: `{:?} == {:?}`\n", $e1, $e2);
            *crate::tests::TEST_FAIL.get() = true;
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

    println!("Running {} tests...", kernel_tests.len());

    for test in kernel_tests {
        print!("Running {}::{} ", test.module, test.name);

        (test.test_fn)();

        print!("\nResult {}::{} ", test.module, test.name);
        if *TEST_FAIL.get() == true {
            print!("[FAIL]\n");
        } else {
            print!("[SUCCESS]\n");
        }

        *TEST_FAIL.get() = false;
    }
}
