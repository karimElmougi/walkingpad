use walkingpad_btle::{WalkingPadReceiver, WalkingPadSender};
use walkingpad_protocol::response::StoredStats;
use walkingpad_protocol::{Mode, Request, Response};

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
    rt.block_on(run())?;

    Ok(())
}

async fn run() -> walkingpad_btle::Result<()> {
    let (sender, receiver) = walkingpad_btle::connect().await?;

    sender.send(&Request::set().mode(Mode::Manual)).await?;

    let stats = fetch_stats(&sender, &receiver).await?;
    println!("fetched stored stats:");
    for stat in stats {
        println!("    {:?}", stat);
    }

    tokio::spawn(async move {
        while let Some(response) = receiver.recv() {
            println!("{:?}", response);
        }
    });

    sender.send(&Request::get().settings()).await?;

    loop {
        sender.send(&Request::get().state()).await?;
    }
}

async fn fetch_stats(
    sender: &WalkingPadSender,
    receiver: &WalkingPadReceiver,
) -> walkingpad_btle::Result<Vec<StoredStats>> {
    sender.send(&Request::get().latest_stored_stats()).await?;

    let mut stats = vec![];
    while let Some(response) = receiver.recv() {
        match response {
            Response::StoredStats(stored_stats) => {
                stats.push(stored_stats);
                if let Some(next_id) = stored_stats.next_id {
                    sender.send(&Request::get().stored_stats(next_id)).await?;
                } else {
                    sender.send(&Request::clear_stats()).await?;
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
