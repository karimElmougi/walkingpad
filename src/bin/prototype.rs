use walkingpad_rs::*;

use std::error::Error;
use std::time::Duration;

use btleplug::api::Manager as _;
use btleplug::api::Peripheral as _;
use btleplug::api::{Central, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};

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
    let control_characteristic = characteristics
        .iter()
        .find(|c| c.uuid == TREADMILL_CHARACTERISTIC_UUID)
        .unwrap();

    println!("Starting walkingpad");

    walkingpad.write(&control_characteristic, &Command::Start.as_bytes(), WriteType::WithoutResponse).await.unwrap();
    tokio::time::sleep(Duration::from_secs(5)).await;

    walkingpad.write(&control_characteristic, &Command::Stop.as_bytes(), WriteType::WithoutResponse).await.unwrap();
    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
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

