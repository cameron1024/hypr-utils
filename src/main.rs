use std::io::{stdout, Write};

use args::{Cmd, JsonFormat, Options};
use clap::Parser;
use serde_json::Value;

mod args;
mod store;
mod system;

const APP_NAME: &str = "hypr-utils";

struct Context<W: Write> {
    opts: Options,
    out: W,
}

impl<W: Write> Context<W> {
    fn write_json(&mut self, value: &Value) -> std::io::Result<()> {
        match self.opts.json_format {
            JsonFormat::Accurate => write!(&mut self.out, "{value}"),
            JsonFormat::Convenient => match value {
                Value::String(s) => write!(&mut self.out, "{s}"),
                other => write!(&mut self.out, "{other}"),
            },
        }
    }

    fn writeln(&mut self) -> std::io::Result<()> {
        writeln!(&mut self.out)
    }
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
