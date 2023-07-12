use anyhow::{anyhow, Context, Result};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout, Duration};
use tokio_util::codec::{Framed, LinesCodec};

#[derive(Clone, Deserialize, Debug)]
pub struct Descriptor {
    pub address: String,
    pub port: usize,
}

impl Descriptor {
    pub fn resource(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }
}

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
        self.send("C1:OFST -15V").await?;

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

    pub async fn read_psu_voltage(&mut self) -> Result<f64> {
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

pub async fn update_voltage_periodically(descriptor: Descriptor, voltage: Arc<Mutex<Option<f64>>>) {
    const UPDATE_PERIOD_MS: Duration = Duration::from_millis(400);

    let mut scope = Scope::open(&descriptor.resource()).await;
    scope.init().await.expect("BUG: Failed to initialize scope");

    loop {
        let voltage_reading = scope.read_psu_voltage().await.ok();
        {
            let mut voltage = voltage.lock().expect("BUG: Failed to acquire voltagelock");
            *voltage = voltage_reading;
        }
        sleep(UPDATE_PERIOD_MS).await;
    }
}
