use walkingpad_protocol::Response;

use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::Peripheral as _;
use btleplug::api::{Central, ScanFilter, WriteType};
use btleplug::api::{Characteristic, Manager as _};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConnectionError(String),
    NoAdapters,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;

        match self {
            ConnectionError(inner) => write!(f, "Error connecting to the WalkingPad: {}", inner),
            NoAdapters => write!(f, "No bluetooth adapters found"),
        }
    }
}

impl std::error::Error for Error {}

impl From<btleplug::Error> for Error {
    fn from(err: btleplug::Error) -> Self {
        Error::ConnectionError(format!("{}", err))
    }
}

const SERVICE_UUID: Uuid = uuid_from_u16(0xfe00);

const READ_CHARACTERISTIC_UUID: Uuid = uuid_from_u16(0xfe01);

const WRITE_CHARACTERISTIC_UUID: Uuid = uuid_from_u16(0xfe02);

#[derive(Debug)]
pub struct WalkingPadReceiver {
    inner: Receiver<Response>,
}

impl WalkingPadReceiver {
    pub fn recv(&self) -> Option<Response> {
        self.inner.recv().ok()
    }
}

#[derive(Debug)]
pub struct WalkingPadSender {
    walkingpad: Peripheral,
    write_characteristic: Characteristic,
    last_write: RefCell<Instant>,
}

impl WalkingPadSender {
    pub async fn send(&self, command: &[u8]) -> Result<()> {
        const MIN_WAIT: Duration = Duration::from_millis(250);
        let time_since_write = self.last_write.borrow().elapsed();
        if time_since_write < MIN_WAIT {
            tokio::time::sleep(MIN_WAIT - time_since_write).await;
        }

        self.walkingpad
            .write(
                &self.write_characteristic,
                command,
                WriteType::WithoutResponse,
            )
            .await?;
        self.last_write.replace(Instant::now());

        Ok(())
    }
}

pub async fn connect() -> Result<(WalkingPadSender, WalkingPadReceiver)> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let main_adapter = adapters.first().ok_or(Error::NoAdapters)?;

    let walkingpad = discover_walkingpad(main_adapter).await?;
    walkingpad.connect().await?;
    walkingpad.discover_services().await?;

    let characteristics = walkingpad.characteristics();

    let write_characteristic = characteristics
        .iter()
        .find(|c| c.uuid == WRITE_CHARACTERISTIC_UUID)
        .ok_or_else(|| Error::ConnectionError("No write characteristic found".to_string()))?
        .clone();

    let read_characteristic = characteristics
        .iter()
        .find(|c| c.uuid == READ_CHARACTERISTIC_UUID)
        .ok_or_else(|| Error::ConnectionError("No read characteristic found".to_string()))?
        .clone();

    let (sender, receiver) = mpsc::channel();
    {
        let walkingpad = walkingpad.clone();
        walkingpad.subscribe(&read_characteristic).await?;
        let mut stream = walkingpad.notifications().await?;
        tokio::spawn(async move {
            while let Some(data) = stream.next().await {
                match Response::deserialize(data.value.as_slice()) {
                    Ok(response) => {
                        if sender.send(response).is_err() {
                            return;
                        }
                    }
                    Err(err) => log::error!("malformed response: {}: `{:?}`", err, data),
                }
            }
        });
    }

    let sender = WalkingPadSender {
        walkingpad,
        write_characteristic,
        last_write: RefCell::new(Instant::now()),
    };

    let receiver = WalkingPadReceiver { inner: receiver };

    Ok((sender, receiver))
}

async fn discover_walkingpad(adapter: &Adapter) -> Result<Peripheral> {
    let filter = ScanFilter {
        services: vec![SERVICE_UUID],
    };
    adapter.start_scan(filter).await?;

    tokio::time::sleep(Duration::from_millis(250)).await;

    let peripherals = match adapter.peripherals().await {
        Ok(p) => p,
        Err(err) => {
            adapter.stop_scan().await?;
            return Err(Error::from(err));
        }
    };

    for peripheral in peripherals {
        let properties = match peripheral.properties().await {
            Ok(p) => p,
            Err(err) => {
                adapter.stop_scan().await?;
                return Err(Error::from(err));
            }
        };

        if let Some(properties) = properties {
            let names = properties.local_name;
            if names.iter().any(|name| name == "WalkingPad") {
                return Ok(peripheral);
            }
        };
    }

    Err(Error::ConnectionError("No WalkingPad found".to_string()))
}
