use super::{Command, Enviroment, COMMANDS};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use rtl::error::ErrorType;
use socket::icmp::{EchoRequest, Icmp};
use socket::socket;

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
        let request = EchoRequest::new(1, 1);

        sock.send_to(
            args[0].try_into().map_err(|_| ErrorType::InvalidArgument)?,
            &request,
            &[],
        )
        .await
        .unwrap();

        let mut req = EchoRequest::new(0, 0);
        let (_, addr) = sock.recv_from(&mut req, &mut []).await?;

        Ok(format!("Got reply from {:?}", addr))
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
