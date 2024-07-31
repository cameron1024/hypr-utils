use battery::units::ratio::ratio;

use crate::{args::SystemCmd, Context};
use std::{error::Error, io::Write};

pub fn handle_cmd<W: Write>(mut ctx: Context<W>, cmd: SystemCmd) -> Result<(), Box<dyn Error>> {
    match cmd {
        SystemCmd::Battery {
            num_spaces,
            override_percentage,
            override_charging,
        } => battery(
            &mut ctx.out,
            num_spaces,
            override_percentage,
            override_charging,
        )?,
    }

    Ok(())
}

fn battery<W: Write>(
    mut out: W,
    num_spaces: u32,
    override_percentage: Option<u8>,
    override_charging: Option<bool>,
) -> Result<(), Box<dyn Error>> {
    let manager = battery::Manager::new()?;
    let battery = manager.batteries()?.next().unwrap().unwrap();

    let charge_level = override_percentage.map(|i| i as i32).unwrap_or_else(|| {
        let charge_level = battery.state_of_charge().get::<ratio>();
        (charge_level * 100.0) as i32
    });

    let is_charging =
        override_charging.unwrap_or_else(|| battery.state() == battery::State::Charging);

    // format-icons = ["" "" "" "" ""];
    let icon = match (is_charging, charge_level) {
        (true, _) => "",
        (false, 0..20) => "",
        (false, 20..40) => "",
        (false, 40..60) => "",
        (false, 60..80) => "",
        (false, 80..=100) => "",
        _ => unreachable!(),
    };

    write!(out, "{icon}")?;
    for _ in 0..num_spaces {
        write!(out, " ")?;
    }

    writeln!(out, "{charge_level}%")?;

    Ok(())
}
