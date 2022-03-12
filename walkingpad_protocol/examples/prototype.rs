use walkingpad_protocol::response::StoredStats;
use walkingpad_protocol::{Request, Response};

use std::error::Error;
use std::time::Duration;

use btleplug::api::bleuuid::uuid_from_u16;
use btleplug::api::Peripheral as _;
use btleplug::api::{Central, ScanFilter, WriteType};
use btleplug::api::{Characteristic, Manager as _};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use uuid::Uuid;

pub const WALKINGPAD_SERVICE_UUID: Uuid = uuid_from_u16(0xfe00);

pub const TREADMILL_READ_CHARACTERISTIC_UUID: Uuid = uuid_from_u16(0xfe01);

pub const TREADMILL_CHARACTERISTIC_UUID: Uuid = uuid_from_u16(0xfe02);

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

    // println!("Waking up");
    // send(
    //     Request::SetMode(Mode::Manual),
    //     &walkingpad,
    //     write_characteristic,
    // )
    // .await;
    // tokio::time::sleep(Duration::from_secs(1)).await;

    println!("Subscribing");
    walkingpad.subscribe(read_characteristic).await.unwrap();

    println!("Fetching past stats");
    let stats = get_stats(&walkingpad, write_characteristic).await;
    for stats in stats {
        println!("{:?}", stats);
    }

    tokio::spawn({
        let walkingpad = walkingpad.clone();
        async move {
            let mut stream = walkingpad.notifications().await.unwrap();
            while let Some(data) = stream.next().await {
                match Response::parse(data.value.as_slice()) {
                    Ok(response) => println!("{:?}", response),
                    Err(err) => println!("err: {}, data: {:?}", err, data.value),
                }
            }
        }
    });

    tokio::time::sleep(Duration::from_secs(5)).await;
    send(
        &Request::get().settings(),
        &walkingpad,
        write_characteristic,
    )
    .await;

    tokio::time::sleep(Duration::from_secs(5)).await;
    send(&Request::get().state(), &walkingpad, write_characteristic).await;

    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
}

async fn send(command: &[u8], walkingpad: &Peripheral, write_characteristic: &Characteristic) {
    walkingpad
        .write(write_characteristic, command, WriteType::WithoutResponse)
        .await
        .unwrap()
}

async fn get_stats(
    walkingpad: &Peripheral,
    write_characteristic: &Characteristic,
) -> Vec<StoredStats> {
    let mut stream = walkingpad.notifications().await.unwrap();

    tokio::time::sleep(Duration::from_secs(1)).await;
    send(
        &Request::get().latest_stored_stats(),
        &walkingpad,
        write_characteristic,
    )
    .await;

    let mut stats = vec![];
    while let Some(data) = stream.next().await {
        match Response::parse(data.value.as_slice()) {
            Ok(Response::StoredStats(stored_stats)) => {
                stats.push(stored_stats);
                if let Some(next_id) = stored_stats.next_id {
                    send(
                        &Request::get().stored_stats(next_id),
                        &walkingpad,
                        &write_characteristic,
                    )
                    .await;
                } else {
                    send(&Request::clear_stats(), &walkingpad, &write_characteristic).await;

                    return stats;
                }
            }
            Err(err) => {
                println!("err: {}, data: {:?}", err, data.value);
                continue;
            }
            _ => {
                continue;
            }
        }
    }

    stats
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
