use walkingpad_btle::{WalkingPadReceiver, WalkingPadSender};
use walkingpad_protocol::request;
use walkingpad_protocol::response::StoredStats;
use walkingpad_protocol::{Mode, Response};

use simplelog::*;
use tokio::runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    let rt = runtime::Builder::new_multi_thread().enable_time().build()?;
    let (sender, receiver) = rt.block_on(walkingpad_btle::connect())?;

    sender.send(request::set::mode(Mode::Manual))?;

    let stats = fetch_stats(&sender, &receiver)?;
    println!("fetched stored stats:");
    for stat in stats {
        println!("    {:?}", stat);
    }

    std::thread::spawn(move || {
        while let Some(response) = receiver.recv() {
            println!("{:?}", response);
        }
    });

    sender.send(request::get::settings())?;

    loop {
        sender.send(request::get::state())?;
    }
}

fn fetch_stats(
    sender: &WalkingPadSender,
    receiver: &WalkingPadReceiver,
) -> walkingpad_btle::Result<Vec<StoredStats>> {
    sender.send(request::get::latest_stored_stats())?;

    let mut stats = vec![];
    while let Some(response) = receiver.recv() {
        match response {
            Response::StoredStats(stored_stats) => {
                stats.push(stored_stats);
                if let Some(next_id) = stored_stats.next_id {
                    sender.send(request::get::stored_stats(next_id))?;
                } else {
                    sender.send(request::clear_stats())?;
                    return Ok(stats);
                }
            }
            other => {
                log::warn!("Other response received while fetching stats: {:?}", other);
                continue;
            }
        }
    }

    Ok(stats)
}
