use battery::units::ratio::ratio;

use crate::{args::SystemCmd, Context};
use std::{error::Error, io::Write};

pub fn handle_cmd<W: Write>(mut ctx: Context<W>, cmd: SystemCmd) -> Result<(), Box<dyn Error>> {
    match cmd {
        SystemCmd::Battery => battery(&mut ctx.out)?,
    }

    Ok(())
}

fn battery<W: Write>(mut out: W) -> Result<(), Box<dyn Error>> {
    let manager = battery::Manager::new()?;
    let battery = manager.batteries()?.next().unwrap().unwrap();

    let percent = battery.state_of_charge().get::<ratio>();
    let is_charging = battery.state() == battery::State::Charging;

    // format-icons = ["" "" "" "" ""];
    let icon = match (is_charging, percent) {
        (true, _) => "",
        (false, 0.0..20.0) => "",
        (false, 20.0..40.0) => "",
        (false, 40.0..60.0) => "",
        (false, 60.0..80.0) => "",
        (false, 80.0..100.0) => "",
        _ => unreachable!(),
    };

    writeln!(out, "{icon} {}%", (percent * 100.0).ceil() as i32)?;

    Ok(())
}
