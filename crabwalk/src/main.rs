use std::fs::File;
use std::io::Write;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use walkingpad_btle::{WalkingPadReceiver, WalkingPadSender};
use walkingpad_protocol::request;
use walkingpad_protocol::response::{MotorState, Response, State, StoredStats};
use walkingpad_protocol::{Mode, Units};

use chrono::{DateTime, Local};
use simplelog::*;

#[derive(Serialize, Deserialize)]
struct RunStats {
    start_time: DateTime<Local>,

    #[serde(with = "humantime_serde")]
    duration: Duration,

    distance: u32,

    nb_steps: u32,
}

impl From<StoredStats> for RunStats {
    fn from(stats: StoredStats) -> RunStats {
        let now = SystemTime::now();
        let time = now - stats.duration;
        let time = DateTime::from(time);
        RunStats {
            start_time: time,
            duration: stats.duration,
            distance: stats.distance,
            nb_steps: stats.nb_steps,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    run()
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = connect_with_retry()?;

    sender.send(request::set::mode(Mode::Manual))?;
    sender.send(request::set::units(Units::Metric))?;

    let mut stats_file = File::options()
        .create(true)
        .append(true)
        .open("stats.json")?;

    let (stats, err) = walkingpad_btle::gather_run_statistics(&sender, &receiver);

    for stats in stats.into_iter().map(RunStats::from) {
        if let Err(err) = serde_json::to_string(&stats).map(|s| writeln!(stats_file, "{}", s)) {
            log::error!("unable to save stored statistics: {}", err);
        }
    }

    if let Some(err) = err {
        return Err(err.into());
    }

    {
        let sender = sender.clone();

        std::thread::spawn(move || {
            let mut app_state: Option<(State, SystemTime)> = None;

            while let Ok(response) = receiver.recv() {
                match response {
                    Response::State(state) => {
                        handle_state_update(&mut app_state, state, &mut stats_file, &sender);
                    }
                    _ => log::info!("{}", response),
                }
            }
        });
    }

    sender.send(request::get::settings())?;

    loop {
        sender.send(request::get::state())?;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn connect_with_retry() -> walkingpad_btle::Result<(WalkingPadSender, WalkingPadReceiver)> {
    let mut retry_count = 0;
    loop {
        let result = walkingpad_btle::connect();
        if let Err(walkingpad_btle::Error::NoWalkingPadFound) = result {
            if retry_count > 3 {
                return result;
            }
            retry_count += 1;
            log::info!("No WalkingPad found, retrying");
            continue;
        }
        return result;
    }
}

fn handle_state_update(
    app_state: &mut Option<(State, SystemTime)>,
    state: State,
    stats_file: &mut File,
    sender: &WalkingPadSender,
) {
    if let Some((last_state, start_time)) = app_state.as_mut() {
        if state.motor_state == MotorState::Running {
            *last_state = state;
            log::info!("{}", last_state);
        } else {
            let stats = RunStats {
                start_time: DateTime::from(*start_time),
                duration: last_state.run_time,
                distance: last_state.distance,
                nb_steps: last_state.nb_steps,
            };

            writeln!(stats_file, "{}", serde_json::to_string(&stats).unwrap()).unwrap();
            let _ = stats_file.flush();
            *app_state = None;
            log::info!("Run finished!");
            let _ = sender.send(request::clear_stats());
        }
    } else if state.motor_state == MotorState::Running {
        log::info!("Run started!");
        *app_state = Some((state, SystemTime::now()));
    }
}
