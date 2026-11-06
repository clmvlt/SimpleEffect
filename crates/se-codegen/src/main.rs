//! # se-codegen
//!
//! Development tool that exports every Rust type deriving `ts_rs::TS` and reachable from the
//! protocol roots to TypeScript, one file per type plus an `index.ts` (ADR-0010, ADR-0012).
//!
//! Usage: `se-codegen <output directory>`. The directory must be empty or absent. Run it through
//! `pnpm codegen` (`scripts/codegen.mjs`), which generates into a scratch folder, then replaces
//! `packages/protocol/src/generated/` (or, with `--check`, compares the two).
//!
//! Not part of the shipped app: nothing depends on this crate.

mod bindings;

use std::{
    env,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

fn main() -> ExitCode {
    let Some(output_dir) = env::args_os().nth(1).map(PathBuf::from) else {
        report("usage: se-codegen <output directory> (normally run through `pnpm codegen`)");
        return ExitCode::FAILURE;
    };
    match bindings::generate(&output_dir) {
        Ok(files) => {
            report(&format!(
                "se-codegen: wrote {} TypeScript files to {}",
                files.len() + 1,
                output_dir.display()
            ));
            ExitCode::SUCCESS
        }
        Err(error) => {
            report(&format!("se-codegen: {error}"));
            ExitCode::FAILURE
        }
    }
}

/// Writes a line on stderr: stdout stays empty so callers can rely on the exit code alone.
fn report(line: &str) {
    // Nothing useful can be done if stderr itself is gone.
    let _ = writeln!(io::stderr().lock(), "{line}");
}
