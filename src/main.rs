use std::io::{stdout, Write};

use args::{Cmd, Options};
use clap::Parser;

mod args;
mod store;

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
        Cmd::Store(store_cmd) => store::handle_cmd(ctx, store_cmd),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }
}
