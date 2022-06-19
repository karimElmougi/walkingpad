use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use walkingpad_btle::{WalkingPadReceiver, WalkingPadSender};
use walkingpad_protocol::request;
use walkingpad_protocol::{Mode, Response};

use simplelog::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    let (sender, receiver) = walkingpad_btle::connect()?;

    sender.send(request::set::mode(Mode::Manual))?;

    fetch_stats(&sender, &receiver)?;

    std::thread::spawn(move || {
        while let Ok(response) = receiver.recv() {
            log::info!("{:?}", response);
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
) -> walkingpad_btle::Result<()> {
    sender.send(request::get::latest_stored_stats())?;

    loop {
        match receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(response) => match response {
                Response::StoredStats(stored_stats) => {
                    log::info!("fetched {:?}", stored_stats);

                    if let Some(next_id) = stored_stats.next_id {
                        sender.send(request::get::stored_stats(next_id))?;
                    } else {
                        sender.send(request::clear_stats())?;
                        return Ok(());
                    }
                }
                other => {
                    log::warn!("Other response received while fetching stats: {:?}", other);
                }
            },
            Err(RecvTimeoutError::Timeout) => {
                log::warn!("recv() timed out, trying again");
                sender.send(request::get::latest_stored_stats())?;
            }
            Err(_) => {
                return Err(walkingpad_btle::Error::ConnectionClosed);
            }
        }
    }
}
