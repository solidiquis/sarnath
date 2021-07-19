# Sarnath

Wraps around a long-running process such as a web-server, watches for changes in the specified directory,
and kills and restarts the process when there is a change. A custom script/command can also be executed onchange.


## Usage

To build, make sure `cargo` and Rust are installed, `git clone` this repo and execute `cargo build --release`.

Given the following Bash scripts:

`ongoing.sh`
```bash

while True; do
  echo "Running.."
  sleep 1
done
```

`onchange.sh`
```bash
echo "File change detected.
```

Sarnath can be executed via:
```
$ ./sarnath -p "bash ongoing.sh" -c "bash onchange.sh"
```

Demo:

<img src="https://github.com/solidiquis/solidiquis/blob/master/assets/sarnath.gif">

## More information:

```
Sarnath 0.1.0
Benjamin Nguyen <benjamin.van.nguyen@gmail.com>

Monitors changes in specified directory and kills and re-executes child process.
Hook in custom command to execute on changes as well.

USAGE:
    sarnath [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --cmd <command>      Command to execute when a file is modified.
    -d, --dir <directory>    Directory to watch. Default is current working directory.
    -p, --proc <process>     Command to boot process that is killed and restarted on-change.
```
