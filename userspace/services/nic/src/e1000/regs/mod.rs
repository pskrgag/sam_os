macro_rules! reg_mod {
    ($md:ident) => {
        pub mod $md;
        pub use $md::*;
    };
}

reg_mod!(ctrl);
reg_mod!(ims);
reg_mod!(regs);
reg_mod!(rctl);
reg_mod!(tctl);
reg_mod!(status);
reg_mod!(rdesc);
reg_mod!(tdesc);
