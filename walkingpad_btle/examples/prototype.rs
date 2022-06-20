use std::fs::File;
use std::io::Write;

use walkingpad_protocol::Mode;
use walkingpad_protocol::{request, Units};

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
    sender.send(request::set::units(Units::Metric))?;

    let mut out = File::options()
        .create(true)
        .write(true)
        .append(true)
        .open("stats.json")?;

    let (mut stats, err) = walkingpad_btle::gather_run_statistics(&sender, &receiver);
    stats.sort_by(|a, b| a.start_time.cmp(&b.start_time));
    for stats in stats {
        if let Err(err) = serde_json::to_string(&stats).map(|s| writeln!(out, "{}", s)) {
            log::error!("unable to save stored statistics: {}", err);
        }
    }

    drop(out);

    if let Some(err) = err {
        return Err(err.into());
    }

    std::thread::spawn(move || {
        while let Ok(response) = receiver.recv() {
            log::info!("{}", response);
        }
    });

    sender.send(request::get::settings())?;

    loop {
        sender.send(request::get::state())?;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
