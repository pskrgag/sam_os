macro_rules! reg_mod {
    ($md:ident) => {
        pub mod $md;
        pub use $md::*;
    };
}

reg_mod!(ctrl);
reg_mod!(regs);
reg_mod!(rctl);
reg_mod!(tctl);
