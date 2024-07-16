use std::path::PathBuf;
use std::env;

use getopts::Options;
use xattr;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} BINARY [options]", program);
    print!("{}", opts.usage(&brief));
}

#[derive(Clone)]
struct Flag<'a> {
    disable: char,
    enable: char,
    help: &'a str,
}

const USER_PAX_FLAGS: &str = "user.pax.flags";
const FLAGTABLE: [Flag; 5] = [
    Flag {disable: 'p', enable: 'P', help: "PAGEEXEC"},
    Flag {disable: 'e', enable: 'E', help: "EMUTRAMP"},
    Flag {disable: 'm', enable: 'M', help: "MPROTECT"},
    Flag {disable: 'r', enable: 'R', help: "RANDMMAP"},
    Flag {disable: 's', enable: 'S', help: "SEGMEXEC"},
];

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut flags: String = "".to_string();

    let mut opts = Options::new();

    for flag in FLAGTABLE.iter() {
        opts.optflag(&flag.disable.to_string(), "", format!("disable {}", flag.help).as_str());
        opts.optflag(&flag.enable.to_string(), "", format!("enable {}", flag.help).as_str());
    }

    opts.optflag("h", "help", "display help message");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!("{}", f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    for flag in FLAGTABLE.iter() {
        if matches.opt_present(flag.disable.to_string().as_str()) {
            flags.push_str(flag.disable.to_string().as_str());
            continue;
        }

        flags.push_str(flag.enable.to_string().as_str());
    }

    if matches.free.is_empty() {
        print_usage(&program, opts);
        return;
    }

    let target_binary = PathBuf::from(matches.free[0].clone());

    match xattr::set(target_binary, USER_PAX_FLAGS, flags.as_bytes()) {
        Ok(()) => {},
        Err(f) => { panic!("setting xattr {}: {}", USER_PAX_FLAGS, f.to_string()) }
    }
}
