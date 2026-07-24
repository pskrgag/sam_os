use super::{Command, Enviroment, COMMANDS};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use rtl::error::ErrorType;

struct Touch;

impl Touch {
    async fn run_internal<'async_trait>(
        &self,
        args: Vec<&str>,
        env: Enviroment<'async_trait>,
    ) -> Result<String, ErrorType> {
        if args.len() == 0 {
            return Err(ErrorType::InvalidArgument);
        }

        env.cwd.OpenFile(args[0].try_into().unwrap(), 1).await?;
        Ok(String::new())
    }
}

#[async_trait::async_trait]
impl Command for Touch {
    fn name(&self) -> &str {
        "touch"
    }

    // TODO: actually walk the dir
    async fn run(&self, args: Vec<&str>, env: Enviroment<'async_trait>) -> Result<String, String> {
        match self.run_internal(args, env).await {
            Ok(s) => Ok(s),
            Err(err) => {
                let s: &str = err.into();

                Err(String::from(s))
            }
        }
    }
}

#[linkme::distributed_slice(COMMANDS)]
static TOUCH: &dyn Command = &Touch;
