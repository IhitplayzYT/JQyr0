use std::error::Error;

use crate::{cmd::{check::check_main, creator::init_main, packer::packer_main, recreate::recreate_main}, helper::Helper::{CLI, Help, Tool}};
mod cmd;
mod helper;
mod config;

fn main() -> Result<(),Box<dyn Error>>{
    let mut clargs = CLI::new();
    clargs.Parse_Args();
    match clargs.subtool{
        Tool::Check => check_main(clargs.tool_args),
        Tool::Packer => packer_main(clargs.tool_args),
        Tool::Finalise => recreate_main(clargs.tool_args),
        Tool::Init => init_main(clargs.tool_args),
        Tool::Help => { Help(None); Ok(()) }
    }
}
