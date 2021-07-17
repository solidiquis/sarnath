mod errors;
mod event;
mod fswatch;
mod signals;

use clap::{Arg, App};
use errors::Error;
use event::Event;
use fswatch::FsWatch;
use std::thread;
use std::sync::mpsc;

const APP_NAME: &'static str = "Sarnath";
const VERSION: &'static str = "0.1.0";
const AUTHOR: &'static str = "Benjamin Nguyen <benjamin.van.nguyen@gmail.com>";
const ABOUT: &'static str = "
About the program.
";

fn main() {
    let matches = App::new(APP_NAME)
        .version(VERSION)
        .author(AUTHOR)
        .about(ABOUT)
        .arg(Arg::with_name("directory")
            .short("d")
            .long("dir")
            .takes_value(true)
            .help("Directory to watch. Default is current working directory.")
            )
        .arg(Arg::with_name("command")
            .short("c")
            .long("cmd")
            .takes_value(true)
            .help("Command to execute when a file is modified.")
            )
        .arg(Arg::with_name("process")
            .short("p")
            .long("proc")
            .takes_value(true)
            .help("Process to rebuild upon file change.")
            )
        .get_matches();

    let dirpath = matches.value_of("directory").unwrap_or(".").clone().to_string();

    let cmd = match matches.value_of("command") {
        None => panic!("{}", Error::MissingArg(String::from("cmd"))),
        Some(c) => c 
    };

    let proc = match matches.value_of("process") {
        None => panic!("{}", Error::MissingArg(String::from("proc"))),
        Some(p) => p
    };

    let mut proc_event = Event::build(proc);
    let mut onchange_event = Event::build(cmd);

    let (fsmod_tx, fsmod_rx) = mpsc::channel();
    let (proc_id_tx, proc_id_rx) = mpsc::channel();
    let (proc_proceed_tx, proc_proceed_rx) = mpsc::channel();

    let mut fs_watch = match FsWatch::new(dirpath, fsmod_tx.clone()) {
        Ok(f) => f,
        Err(e) => panic!("Failed to initialize FsWatch with error: {}", e)
    };

    let mut threads = vec![];

    let poll = thread::spawn(move || {
        fs_watch.poll()
    });

    threads.push(poll);
    
    let proc_loop = thread::spawn(move || {
        proc_event.init_proc_loop(proc_proceed_rx, proc_id_tx)
    });

    threads.push(proc_loop);

    let onchange_event = thread::spawn(move || {
        onchange_event.listen_for_change(proc_proceed_tx, proc_id_rx, fsmod_rx)
    });

    threads.push(onchange_event);

    for t in threads {
        t.join().unwrap()
    } 
}
