use std::path::PathBuf;
use xshell::{cmd, pushd, read_file, rm_rf};

use anyhow::{anyhow, Context, Result};
use pico_args::Arguments;

fn main() -> Result<()> {
    let mut args = Arguments::from_env();
    let subcmd = args.subcommand()?.unwrap_or_default();

    goto_root()?;

    match subcmd.as_str() {
        "review" => {
            review()?;
        }
        _ => eprintln!("cargo xtask codegen"),
    }

    Ok(())
}

fn review() -> Result<()> {
    use crossterm::event::{self, Event, KeyCode, KeyEvent};
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

    for file in glob::glob("./tests/**/*.new")? {
        let filename = file?;
        let dir = filename
            .parent()
            .context("Could not find directory for .new")?;
        let file = read_file(&filename)?;
        println!("Accepting: {:?}", &filename);
        println!("-----");
        let diff = cmd!("colordiff").stdin(file.as_bytes()).read()?;

        println!("{}\n\n", diff);
        println!("-----");
        loop {
            println!("[Aa]ccept, [Rr]eject, [Ss]kip or [Qq]uit");

            enable_raw_mode()?;
            let event = event::read()?;
            disable_raw_mode()?;
            if let Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) = event
            {
                match c {
                    'a' | 'A' => {
                        let _p = pushd(dir);

                        cmd!("patch --ignore-whitespace")
                            .stdin(file.as_bytes())
                            .read()?;

                        break;
                    }
                    'r' | 'R' => {
                        println!("Rejecting change");
                        rm_rf(&filename)?;
                        break;
                    }
                    's' | 'S' => {
                        println!("Skipping");
                        break;
                    }
                    'q' | 'Q' => {
                        println!("Quitting");
                        return Ok(());
                    }
                    _ => continue,
                }
            }
        }
    }
    println!("All processed");
    for file in glob::glob("./tests/**/*.orig")? {
        let filename = file?;
        cmd!("rm {filename}").read()?;
    }
    Ok(())
}

fn goto_root() -> Result<()> {
    let git = PathBuf::from(".git");
    loop {
        if git.exists() {
            break Ok(());
        }
        let cwd = std::env::current_dir()?;
        let parent = cwd
            .parent()
            .ok_or_else(|| anyhow!("Could not find .git root"))?;
        std::env::set_current_dir(parent)?;
    }
}
