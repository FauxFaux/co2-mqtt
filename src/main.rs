use std::io::{Error, Read};
use std::os::unix::io::AsRawFd;
use std::{env, fmt, fs};

use anyhow::{anyhow, bail, Context, Result};
use log::debug;
use rumqttc::{MqttOptions, QoS};
use time::OffsetDateTime;

#[derive(Debug)]
enum Value {
    CO2(u16),
    Temp(f64),
}

impl Value {
    fn name(&self) -> &'static str {
        match self {
            Value::CO2(_) => "co2",
            Value::Temp(_) => "temp",
        }
    }

    fn value(&self) -> f64 {
        match self {
            Value::CO2(x) => f64::from(*x),
            Value::Temp(x) => *x,
        }
    }
}

struct Record {
    op: u8,
    low: u16,
}

impl fmt::Debug for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Record {{ op: 0x{:02x}, val: {:>5} }}",
            self.op, self.low
        )
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Message {
    /// the date/time I believe to be true
    #[serde(with = "time::serde::rfc3339")]
    publish_time: OffsetDateTime,

    value: f64,
}

impl Record {
    fn decode(&self) -> Option<Value> {
        Some(match self.op {
            0x42 => Value::Temp(decode_temp(self.low)),
            0x50 => Value::CO2(self.low),
            _ => return None,
        })
    }
}

fn is_valid_checksum(buf: &[u8; 8]) -> bool {
    let sum: u16 = buf[..3].iter().map(|v| *v as u16).sum();
    let sum = (sum & 0xFF) as u8;
    sum == buf[3]
}

fn parse_data(buf: &[u8; 8]) -> Result<Record> {
    if buf[4] != 0x0D || !is_valid_checksum(&buf) {
        bail!("Invalid bytes were read: {:02x?}", buf);
    }

    let op = buf[0];
    let low = u16::from(buf[1]);
    let low = low << 8 | u16::from(buf[2]);

    Ok(Record { op, low })
}

fn decode_temp(val: u16) -> f64 {
    val as f64 / 16.0 - 273.15
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let device = env_var("CO2_DEVICE")?;
    let device_id = env_var("CO2_NAME").unwrap_or_else(|_| "monitor".to_string());

    let mut f = fs::OpenOptions::new().read(true).write(true).open(device)?;

    let mqtt_opts = MqttOptions::parse_url(env_var("MQTT_URL")?)?;
    let (mut mqtt, _connection) = rumqttc::Client::new(mqtt_opts, 60);

    let request = [0u8; 9];
    let answer = unsafe { ::libc::ioctl(f.as_raw_fd(), 0xC0094806, &request) };
    if answer <= 0 {
        Err(Error::last_os_error()).with_context(|| anyhow!("ioctl failed: {answer}"))?;
    }
    loop {
        let mut buf = [0u8; 8];
        f.read_exact(&mut buf)?;
        let record = parse_data(&buf)?;
        let publish_time = OffsetDateTime::now_utc();
        let (key, value) = match record.decode() {
            Some(named) => (
                named.name().to_string(),
                Message {
                    publish_time,
                    value: named.value(),
                },
            ),
            None => (
                format!("unknown_0x{:02x}", record.op),
                Message {
                    publish_time,
                    value: record.low as f64,
                },
            ),
        };
        let topic = format!("co2/{device_id}/SENSOR/{key}");
        let payload = serde_json::to_string(&value)?;
        debug!("publishing to:{} payload:{}", topic, payload);
        mqtt.publish(topic, QoS::AtLeastOnce, true, payload)?;
    }
}

fn env_var(name: &'static str) -> Result<String> {
    env::var(name).with_context(|| anyhow!("reading env var {name:?}"))
}
