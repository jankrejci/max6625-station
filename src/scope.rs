use anyhow::{anyhow, Context, Result};
use futures::{SinkExt, StreamExt};
use std::str::FromStr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};
use tokio_util::codec::{Framed, LinesCodec};

pub struct Scope {
    control: Framed<TcpStream, LinesCodec>,
}

impl Scope {
    const COMMAND_REPLY_TIMEOUT: Duration = Duration::from_millis(3000);

    pub async fn open(resource_addr: &str) -> Self {
        let tcp_stream = timeout(
            Duration::from_millis(2000),
            TcpStream::connect(resource_addr),
        )
        .await
        .expect("BUG: connection timeout")
        .expect("BUG: Cannot connect to scope");

        let control = Framed::new(tcp_stream, LinesCodec::new());

        Self { control }
    }

    pub async fn init(&mut self) -> Result<()> {
        self.send("*RST").await?;
        // Timebase 1 ms / div
        self.send("TDIV 1MS").await?;
        // Probe attenuation 10x
        self.send("C1:ATTN 10").await?;
        // Channell sensitivity 5 V / div
        self.send("C1:VDIV 5V").await?;
        // Offset -10 V
        self.send("C1:OFST -10V").await?;

        sleep(Duration::from_millis(5000)).await;
        Ok(())
    }

    pub async fn send(&mut self, payload: &str) -> Result<()> {
        self.control.send(payload.to_string()).await?;
        Ok(())
    }

    async fn recv(&mut self) -> Result<String> {
        let response = match timeout(Self::COMMAND_REPLY_TIMEOUT, self.control.next())
            .await
            .context("PSU SCPI waiting response")?
        {
            Some(result) => result.context("PSU SCPI reading response")?,
            None => anyhow::bail!("Unexpected end of PSU control stream"),
        };
        Ok(response)
    }

    pub async fn read_mean(&mut self) -> Result<f64> {
        self.send("C1:PAVA? MEAN").await?;
        let response = self.recv().await?;

        match response.trim().split(',').nth(1) {
            None => Err(anyhow!("Received wrong response")),
            Some(value) => {
                let value = value.replace('V', "");
                Ok(f64::from_str(&value)?)
            }
        }
    }
}
