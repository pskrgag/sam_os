use super::{COMMANDS, Command, Enviroment};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use rtl::error::ErrorType;
use socket::{Icmp, socket};
use net::ipv4::IPv4;
use net::ipv4::icmp::IcmpHeader;

struct Ping;

impl Ping {
    async fn run_internal<'async_trait>(
        &self,
        args: Vec<&str>,
        _env: Enviroment<'async_trait>,
    ) -> Result<String, ErrorType> {
        if args.is_empty() {
            return Err(ErrorType::InvalidArgument);
        }

        let sock = socket::<Icmp>().await?;

        sock.send_to(IPv4::new(1, 2, 3, 4), &IcmpHeader::new(0, 1), &[]).await?;
        todo!()
    }
}

#[async_trait::async_trait]
impl Command for Ping {
    fn name(&self) -> &str {
        "ping"
    }

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
static PING: &dyn Command = &Ping;
