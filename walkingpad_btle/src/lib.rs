use futures::Stream;
use once_cell::sync::OnceCell;
use walkingpad_protocol::{Request, Response};
use walkingpad_protocol::response::raw;

use std::fmt;
use std::fmt::Display;
use std::pin::Pin;
use std::time::{Duration, Instant};

use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::Peripheral as _;
use btleplug::api::{Central, ScanFilter, WriteType};
use btleplug::api::{Manager as _, ValueNotification};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::future;
use futures::stream::StreamExt;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ConnectionError(String),
    ConnectionClosed,
    ConnectionAlreadyEstablished,
    NoAdapters,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;

        match self {
            ConnectionError(inner) => write!(f, "Error connecting to the WalkingPad: {}", inner),
            ConnectionClosed => write!(f, "Connection was closed"),
            ConnectionAlreadyEstablished => write!(f, "Connection was already established"),
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

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::ConnectionError(format!("{}", err))
    }
}

impl From<std::sync::mpsc::SendError<Request>> for Error {
    fn from(_: std::sync::mpsc::SendError<Request>) -> Self {
        Error::ConnectionClosed
    }
}

pub type WalkingPadReceiver = std::sync::mpsc::Receiver<Response>;

pub type WalkingPadSender = std::sync::mpsc::SyncSender<Request>;

static CONNECTION_FLAG: OnceCell<()> = OnceCell::new();

pub fn connect() -> Result<(WalkingPadSender, WalkingPadReceiver)> {
    if CONNECTION_FLAG.get().is_some() {
        return Err(Error::ConnectionAlreadyEstablished);
    }

    let (receiver_in, receiver_out) = std::sync::mpsc::channel();
    let (sender_in, sender_out) = std::sync::mpsc::sync_channel::<Request>(10);
    let (init_in, init_out) = tokio::sync::oneshot::channel::<Result<()>>();

    let _t = std::thread::spawn(move || {
        macro_rules! unwrap_or_return {
            ( $e:expr ) => {
                match $e {
                    Ok(x) => x,
                    Err(err) => {
                        init_in.send(Err(err.into())).unwrap();
                        return;
                    }
                }
            };
        }

        let rt = unwrap_or_return!(tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build());

        let walkingpad = unwrap_or_return!(rt.block_on(init_walkingpad()));

        let mut notification_stream =
            unwrap_or_return!(rt.block_on(notification_stream(walkingpad.clone())));

        let write_characteristic = {
            let characteristics = walkingpad.characteristics();

            const WRITE_CHARACTERISTIC_UUID: Uuid = uuid_from_u16(0xfe02);

            unwrap_or_return!(characteristics
                .iter()
                .find(|c| c.uuid == WRITE_CHARACTERISTIC_UUID)
                .ok_or_else(|| Error::ConnectionError("No write characteristic found".to_string())))
            .clone()
        };

        let sender = async move {
            let mut last_write = Instant::now();

            while let Ok(command) = async { sender_out.recv() }.await {
                const MIN_WAIT: Duration = Duration::from_millis(500);

                let time_since_write = last_write.elapsed();
                if time_since_write < MIN_WAIT {
                    tokio::time::sleep(MIN_WAIT - time_since_write).await;
                }

                let result = walkingpad
                    .write(
                        &write_characteristic,
                        command.as_bytes(),
                        WriteType::WithoutResponse,
                    )
                    .await;

                if let Err(err) = result {
                    log::error!("WalkingPad write failed: {}", err);
                    break;
                }

                last_write = Instant::now();
            }
        };

        let receiver = async move {
            while let Some(data) = notification_stream.next().await {
                match raw::Response::parse(data.value.as_slice()) {
                    Ok(response) => {
                        if receiver_in.send(response.into()).is_err() {
                            break;
                        }
                    }
                    Err(err) => log::error!("malformed response: {}: `{:?}`", err, data),
                }
            }
        };

        init_in.send(Ok(())).unwrap();

        rt.block_on(async { future::join(sender, receiver).await });
    });

    init_out.blocking_recv().unwrap()?;

    CONNECTION_FLAG.set(()).unwrap();

    Ok((sender_in, receiver_out))
}

async fn init_walkingpad() -> Result<Peripheral> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let main_adapter = adapters.first().ok_or(Error::NoAdapters)?;

    let walkingpad = discover_walkingpad(main_adapter).await?;
    walkingpad.connect().await?;
    walkingpad.discover_services().await?;

    Ok(walkingpad)
}

type NotificationStream = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;

async fn notification_stream(walkingpad: Peripheral) -> Result<NotificationStream> {
    let characteristics = walkingpad.characteristics();

    const READ_CHARACTERISTIC_UUID: Uuid = uuid_from_u16(0xfe01);

    let read_characteristic = characteristics
        .iter()
        .find(|c| c.uuid == READ_CHARACTERISTIC_UUID)
        .ok_or_else(|| Error::ConnectionError("No read characteristic found".to_string()))?;

    walkingpad.subscribe(read_characteristic).await?;

    let stream = walkingpad.notifications().await?;

    Ok(stream)
}

async fn discover_walkingpad(adapter: &Adapter) -> Result<Peripheral> {
    const SERVICE_UUID: Uuid = uuid_from_u16(0xfe00);
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
