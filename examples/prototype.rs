use crabwalkpad::*;

use std::error::Error;
use std::time::Duration;

use btleplug::api::Peripheral as _;
use btleplug::api::{Central, ScanFilter, WriteType};
use btleplug::api::{Characteristic, Manager as _};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;

    let adapters = manager.adapters().await?;
    let central = adapters.first().unwrap();

    let walkingpad = discover_walkingpad(central).await;

    walkingpad.connect().await.unwrap();
    println!("Connected");

    walkingpad.discover_services().await.unwrap();
    println!("Dicovered characteristics");

    let characteristics = walkingpad.characteristics();
    let write_characteristic = characteristics
        .iter()
        .find(|c| c.uuid == TREADMILL_CHARACTERISTIC_UUID)
        .unwrap();

    let read_characteristic = characteristics
        .iter()
        .find(|c| c.uuid == TREADMILL_READ_CHARACTERISTIC_UUID)
        .unwrap();

    println!("Waking up");
    send(
        Command::SetMode(Mode::Manual),
        &walkingpad,
        write_characteristic,
    )
    .await;
    tokio::time::sleep(Duration::from_secs(5)).await;

    println!("Subscribing");
    walkingpad.subscribe(read_characteristic).await.unwrap();

    tokio::spawn({
        let walkingpad = walkingpad.clone();
        async move {
            let mut stream = walkingpad.notifications().await.unwrap();
            while let Some(data) = stream.next().await {
                println!("{:?}", data.value);
            }
        }
    });

    println!("Starting");
    send(Command::Start, &walkingpad, write_characteristic).await;
    tokio::time::sleep(Duration::from_secs(10)).await;

    for _ in 0..3 {
        println!("Sending query");
        send(Command::Query, &walkingpad, write_characteristic).await;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    println!("Stopping");
    send(Command::Stop, &walkingpad, write_characteristic).await;
    tokio::time::sleep(Duration::from_secs(5)).await;

    println!("Setting to sleep");
    send(
        Command::SetMode(Mode::Sleep),
        &walkingpad,
        write_characteristic,
    )
    .await;

    Ok(())
}

async fn send(command: Command, walkingpad: &Peripheral, write_characteristic: &Characteristic) {
    walkingpad
        .write(
            write_characteristic,
            &command.as_bytes(),
            WriteType::WithoutResponse,
        )
        .await
        .unwrap()
}

async fn discover_walkingpad(central: &Adapter) -> Peripheral {
    central
        .start_scan(ScanFilter {
            services: vec![WALKINGPAD_SERVICE_UUID],
        })
        .await
        .unwrap();

    loop {
        for p in central.peripherals().await.unwrap() {
            if p.properties()
                .await
                .unwrap()
                .unwrap()
                .local_name
                .iter()
                .any(|name| name == "WalkingPad")
            {
                return p;
            }
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
