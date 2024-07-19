use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use clap::Parser;
use xattr;

const USER_PAX_FLAGS: &str = "user.pax.flags";
const HELP_MSG: &str = "
paxmark - a utility for setting PaX markings on binaries

Usage:
% paxmark -[pP|eE|mM|rR|sS] <binary>

Each letter corresponds to a PaX feature flag, upper case
enabling and lower case disabling.

This utility will clear out invalid marks and explicitly set
enabled for missing marks, matching defaults.

-p|P    PAGEEXEC: use NX-bit to mark unexecutable pages
-e|E    EMUTRAMP: emulate stack trampolines
-m|M    MPROTECT: write-xor-execute in mmap/mprotect(2) syscalls
-r|R    RANDMMAP: address space layout randomization (ASLR)
-s|S    SEGMEXEC: segmentation-based NX-bit emulation
";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_hint = clap::ValueHint::DirPath)]
    binary: PathBuf,
    #[arg(short = 'P', group = "pageexec")]
    e_pageexec: bool,
    #[arg(short = 'p', group = "pageexec")]
    d_pageexec: bool,
    #[arg(short = 'E', group = "emutramp")]
    e_emutramp: bool,
    #[arg(short = 'e', group = "emutramp")]
    d_emutramp: bool,
    #[arg(short = 'M', group = "mprotect")]
    e_mprotect: bool,
    #[arg(short = 'm', group = "mprotect")]
    d_mprotect: bool,
    #[arg(short = 'R', group = "randmmap")]
    e_randmmap: bool,
    #[arg(short = 'r', group = "randmmap")]
    d_randmmap: bool,
    #[arg(short = 'S', group = "segmexec")]
    e_segmexec: bool,
    #[arg(short = 's', group = "segmexec")]
    d_segmexec: bool,
}

impl Cli {
    // We don't need to test this because we test Delta::new
    fn get_delta(&self) -> BTreeMap<char, Delta> {
        BTreeMap::from([
            ('P', Delta::new(self.e_pageexec, self.d_pageexec)),
            ('E', Delta::new(self.e_emutramp, self.d_emutramp)),
            ('M', Delta::new(self.e_mprotect, self.d_mprotect)),
            ('R', Delta::new(self.e_randmmap, self.d_randmmap)),
            ('S', Delta::new(self.e_segmexec, self.d_segmexec)),
        ])
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Delta {
    Enable,
    Disable,
    Keep,
}

impl Delta {
    fn new(enable: bool, disable: bool) -> Self {
        use Delta::*;
        match (enable, disable) {
            (true, _) => Enable,
            (_, true) => Disable,
            _ => Keep,
        }
    }

    fn apply(self, c: char) -> char {
        use Delta::*;
        match self {
            Enable => c.to_ascii_uppercase(),
            Disable => c.to_ascii_lowercase(),
            Keep => c,
        }
    }
}

fn main() {
    // This is easier than properly adding the help message to the derive-based Parser
    use clap::error::ErrorKind as ClapErrKind;
    let cli = match Cli::try_parse() {
        // Happy case with usable args
        Ok(x) => x,
        // Unhappy case that we're fine letting clap handle
        Err(err) if err.kind() != ClapErrKind::DisplayHelp => {
            err.exit();
        }
        _ => {
            // Print the help message by hand
            println!("{HELP_MSG}");
            std::process::exit(0);
        }
    };

    // Get the current xattr value, print it for transparency
    let current = get_value(&cli.binary);
    println!("Current {USER_PAX_FLAGS} xattr value: {current}");

    let mut valid_current = true;

    // Map of marks to if/how to change them, derived from the CLI flags
    let mut delta = cli.get_delta();

    // Iterate over the current state, removing the deltas so we can only have one
    // match per mark.
    //
    // For each match, apply the delta. Each non match is either a duplicate or invalid
    // mark, so the current state is dirty. We'll use the first matches in the value as
    // valid, filtering out the rest.
    //
    // TODO: extract and test against proper and improper current values
    let mut new = current
        .chars()
        .filter_map(|c| match delta.remove(&c.to_ascii_uppercase()) {
            Some(d) => Some(d.apply(c)),
            None => {
                valid_current = false;
                None
            }
        })
        .collect::<String>();

    // The remaining keys are marks that weren't matched in the current xattr value, ergo
    // are missing and should have their defaults added. The keys are capitalised, which
    // means enabled, so we can simply add the keys as the marks.
    for (key, _) in delta.into_iter() {
        new.push(key);
    }

    // Just let the user know, so there are no surprises.
    if !valid_current {
        eprintln!("The old {USER_PAX_FLAGS} value is either dirty or invalid");
        eprintln!("Only the first valid marks from this value will be used");
    }

    // And finally apply the new value
    if let Err(err) = set_value(&cli.binary, &new) {
        eprintln!("Error on setting xattr value: {err}");
    } else {
        println!("Set {USER_PAX_FLAGS} xattr to {new} successfully!");
    }
}

fn get_value(binary: impl AsRef<Path>) -> String {
    // Default to all enabled
    let default = String::from("PEMRS");
    match xattr::get(binary, USER_PAX_FLAGS) {
        Ok(Some(flags)) => String::from_utf8(flags).unwrap_or(default),
        _ => default,
    }
}

fn set_value(binary: impl AsRef<Path>, value: impl AsRef<[u8]>) -> std::io::Result<()> {
    xattr::set(binary, USER_PAX_FLAGS, value.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use Delta::*;

    #[test]
    fn cli_to_flag_delta() {
        // new(enabled, disabled) -> Delta
        // Precedence:
        // - if enabled  -> Enable
        // - if disabled -> Disable
        // - if neither  -> Keep
        assert_eq!(Delta::new(true, true), Enable);
        assert_eq!(Delta::new(true, false), Enable);
        assert_eq!(Delta::new(false, true), Disable);
        assert_eq!(Delta::new(false, false), Keep);
    }

    #[test]
    fn delta_apply() {
        let e = Enable;
        let d = Disable;
        let k = Keep;

        assert_eq!(e.apply('Q'), 'Q');
        assert_eq!(e.apply('q'), 'Q');
        assert_eq!(d.apply('Q'), 'q');
        assert_eq!(d.apply('q'), 'q');
        assert_eq!(k.apply('Q'), 'Q');
        assert_eq!(k.apply('q'), 'q');
    }
}
