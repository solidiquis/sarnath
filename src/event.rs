use crate::signals::Signal;
use crate::errors::Error;
use libc;
use std::{
    io::{self, BufRead},
    process::{self, Command},
    sync::mpsc::{Sender, Receiver},
};

pub struct Event {
    pub cmd: Command
}

// TODO: Need to exit program if any event logs to stderr.

impl Event {
    pub fn build(cmdargs: &str) -> Self {
        let cmdargs: Vec<&str> = cmdargs.split(" ").collect();

        let mut cmd = Command::new(cmdargs[0].clone());

        for i in 1..cmdargs.len() {
            cmd.arg(cmdargs[i].clone());
        }

        Event { cmd }
    }

    pub fn init_proc_loop(
        &mut self,
        proc_proceed_rx: Receiver<Signal>,
        proc_id_tx: Sender<u32>
        ) 
    {
        proc_proceed_rx.recv().unwrap();

        loop {
            let child = self
                .cmd
                .stdout(process::Stdio::piped())
                .spawn()
                .unwrap();

            proc_id_tx.send(child.id()).unwrap();

            let stdout = child
                .stdout
                .ok_or_else(|| panic!("{}", Error::FailedStdoutCapture))
                .unwrap();

            let reader = io::BufReader::new(stdout);

            reader
                .lines()
                .filter_map(|ln| ln.ok())
                .for_each(|ln| println!("{}", ln));

            proc_proceed_rx.recv().unwrap();
        }
    }

    pub fn listen_for_change(
        &mut self,
        proc_proceed_tx: Sender<Signal>,
        proc_id_rx: Receiver<u32>,
        fsmod_rx: Receiver<Signal>
        )
    {
        proc_proceed_tx.send(Signal::ProcProceed).unwrap();

        loop {
            let proc_id = proc_id_rx.recv().unwrap();

            fsmod_rx.recv().unwrap();

            unsafe {
                libc::kill(proc_id as i32, libc::SIGKILL);
            }

            self.cmd.spawn().unwrap();    

            proc_proceed_tx.send(Signal::ProcProceed).unwrap();
        }
    }
}

