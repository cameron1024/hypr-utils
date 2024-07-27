use std::io::{stdout, Write};

use args::{Cmd, Options};
use clap::Parser;

mod args;
mod store;
mod system;

const APP_NAME: &str = "hypr-utils";

struct Context<W: Write> {
    opts: Options,
    out: W,
}

fn main() {
    let args = args::Args::parse();
    let ctx = Context {
        opts: args.opts,
        out: stdout(),
    };

    let result = match args.cmd {
        Cmd::Store(cmd) => store::handle_cmd(ctx, cmd),
        Cmd::System(cmd) => system::handle_cmd(ctx, cmd),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }
}
