use crate::fswatch::FsWatch;
use crate::event::Event;
use crate::signals::Signal;
use std::fs::{self, DirEntry};
use std::process::Stdio;
use std::sync::mpsc;
use std::time::Duration;
use std::thread;
use rand::Rng;

fn mktmpdir<F: Fn(&str)>(uid: u32, f: F) {
    let path = format!("{}-{}", "./tmp", uid);

    let tmpdir = fs::create_dir(&path);
    match tmpdir {
        Err(e) => panic!("{}", e),
        _ => ()
    }

    let mut test_files = Vec::new();
    for i in 0..5 {
        test_files.push(format!("./{}/test_{}.txt", &path, i))
    }

    for file in test_files.iter() {
        fs::File::create(file);
    }

    f(&path);

    fs::remove_dir_all(&path);
}

fn cleanup(path: &str) {
    fs::remove_dir_all(path);
}

#[test]
fn test_fswatch_poll() {
    let mut rng = rand::thread_rng();
    let uid: u32 = rng.gen();

    mktmpdir(uid, |p :&str| {
        let (fsmod_tx, rx) = mpsc::channel();
        let mut fswatch = match FsWatch::new(String::from(p), fsmod_tx) {
            Ok(f) => f,
            Err(e) => {
                cleanup(p);
                panic!("Failed to initialize FsWatch with error: {}", e)
            }
        };

        thread::spawn(move || {
            fswatch.poll()
        });

        fs::File::create(format!("./{}/new.txt", p));

        assert_eq!(rx.recv().unwrap(), Signal::FsMod)
    })
}

#[test]
fn test_event_listen_for_change() {
    let mut rng = rand::thread_rng();
    let uid: u32 = rng.gen();

    mktmpdir(uid, |p :&str| {
        let (proc_proceed_tx, proc_proceed_rx) = mpsc::channel();
        let (proc_id_tx, proc_id_rx) = mpsc::channel();
        let (fsmod_tx, fsmod_rx) = mpsc::channel();

        let mut onchange_event = Event::build("echo Cthulhu");
        onchange_event.cmd.stdout(Stdio::null());

        thread::spawn(move || {
            onchange_event.listen_for_change(
                proc_proceed_tx.clone(),
                proc_id_rx,
                fsmod_rx
                )
        });

        // onchange_event should automatically green-light the 
        // main child process to execute from the get-go.
        match proc_proceed_rx.recv_timeout(Duration::from_millis(1_000)) {
            Err(_) => {
                cleanup(p);
                panic!("Did not receive ProcProceed Signal.")
            },
            _ => ()
        };

        proc_id_tx.send(1).unwrap();

        fsmod_tx.send(Signal::FsMod).unwrap();
        
        assert_eq!(proc_proceed_rx.recv().unwrap(), Signal::ProcProceed)
    })
}

#[test]
fn test_event_init_proc_loop() {
    let mut rng = rand::thread_rng();
    let uid: u32 = rng.gen();

    mktmpdir(uid, |p :&str| {
        let (proc_proceed_tx, proc_proceed_rx) = mpsc::channel();
        let (proc_id_tx, proc_id_rx) = mpsc::channel();

        let mut proc = Event::build("echo Cthulhu");
        proc.cmd.stdout(Stdio::null());

        thread::spawn(move || {
            proc.init_proc_loop(
                proc_proceed_rx,
                proc_id_tx
                )
        });

        // Initialize process and ensure that it sends it's PID to listener.
        proc_proceed_tx.send(Signal::ProcProceed).unwrap();
        match proc_id_rx.recv_timeout(Duration::from_millis(1_000)) {
            Err(_) => panic!("Failed to receive Child's process ID."),
            _ => ()
        }

        // Allow process to re-exec once it sends its PID.
        proc_proceed_tx.send(Signal::ProcProceed).unwrap();
        match proc_id_rx.recv_timeout(Duration::from_millis(1_000)) {
            Err(_) => panic!("Child process failed to re-execute."),
            _ => ()
        }
    })
}
