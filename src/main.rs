mod av;
mod canvas;
mod cmds;
mod opts;

use clap::Clap;

use cmds::*;
use opts::SubCommand::*;

fn main() {
    let opts = opts::Opts::parse();

    match opts.subcmd {
        Forward(args) => forward::run(args),
        UI => ui::run(),
    }
}
