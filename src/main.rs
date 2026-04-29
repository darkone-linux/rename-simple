use rename_files::{compute_renames, RenameOp};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs, process};

// ─────────────────────────────────────────────────────────────────────────────
// CLI
// ─────────────────────────────────────────────────────────────────────────────

fn usage(program: &str) {
    eprintln!("Usage: {program} [OPTIONS] [DIR]");
    eprintln!();
    eprintln!("Rename files in DIR (default: current directory) by normalising");
    eprintln!("accented characters, replacing spaces and special chars with `-`,");
    eprintln!("and lowercasing everything.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -n, --dry-run   Show what would be renamed without touching any file");
    eprintln!("  -h, --help      Print this help message");
    eprintln!();
    eprintln!("Planned (not yet implemented):");
    eprintln!("  -r              Process directories recursively");
}

struct Config {
    dir: PathBuf,
    dry_run: bool,
}

fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    let mut dry_run = false;
    let mut dir: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                usage(program);
                process::exit(0);
            }
            "-n" | "--dry-run" => {
                dry_run = true;
            }
            "-r" => {
                return Err("The -r (recursive) flag is not yet implemented.".to_owned());
            }
            flag if flag.starts_with('-') => {
                return Err(format!("Unknown flag: {flag}"));
            }
            path => {
                if dir.is_some() {
                    return Err("Too many positional arguments.".to_owned());
                }
                dir = Some(PathBuf::from(path));
            }
        }
        i += 1;
    }

    let dir =
        dir.unwrap_or_else(|| env::current_dir().expect("Cannot determine current directory"));

    if !dir.is_dir() {
        return Err(format!("'{}' is not a directory.", dir.display()));
    }

    Ok(Config { dir, dry_run })
}

// ─────────────────────────────────────────────────────────────────────────────
// Conflict detection
// ─────────────────────────────────────────────────────────────────────────────

/// Return ops that are safe to apply (no destination conflicts).
/// Conflicting ops are printed as warnings and dropped.
fn filter_conflicts(ops: Vec<RenameOp>) -> Vec<RenameOp> {
    // Count how many ops share the same destination name
    let mut dest_count: HashMap<PathBuf, usize> = HashMap::new();
    for op in &ops {
        *dest_count.entry(op.to.clone()).or_insert(0) += 1;
    }

    let mut safe = Vec::new();
    for op in ops {
        if dest_count[&op.to] > 1 {
            eprintln!(
                "  ⚠  CONFLICT – skipping '{}': multiple files would rename to '{}'",
                op.from.file_name().unwrap().to_string_lossy(),
                op.to.file_name().unwrap().to_string_lossy(),
            );
        } else if op.to.exists() {
            eprintln!(
                "  ⚠  CONFLICT – skipping '{}': destination '{}' already exists",
                op.from.file_name().unwrap().to_string_lossy(),
                op.to.file_name().unwrap().to_string_lossy(),
            );
        } else {
            safe.push(op);
        }
    }
    safe
}

// ─────────────────────────────────────────────────────────────────────────────
// main
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    let config = match parse_args() {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("Error: {msg}");
            eprintln!("Run with --help for usage.");
            process::exit(1);
        }
    };

    if config.dry_run {
        println!("Dry run – no files will be modified.\n");
    }

    println!("Directory: {}\n", config.dir.display());

    let ops = match compute_renames(&config.dir) {
        Ok(ops) => ops,
        Err(err) => {
            eprintln!("Error reading directory: {err}");
            process::exit(1);
        }
    };

    if ops.is_empty() {
        println!("Nothing to rename.");
        return;
    }

    let ops = filter_conflicts(ops);

    let mut renamed = 0usize;
    let mut errors = 0usize;

    for op in &ops {
        let from_name = op.from.file_name().unwrap().to_string_lossy();
        let to_name = op.to.file_name().unwrap().to_string_lossy();

        if config.dry_run {
            println!("  {} → {}", from_name, to_name);
            renamed += 1;
        } else {
            match fs::rename(&op.from, &op.to) {
                Ok(()) => {
                    println!("  {} → {}", from_name, to_name);
                    renamed += 1;
                }
                Err(e) => {
                    eprintln!("  ✗ Error renaming '{}': {}", from_name, e);
                    errors += 1;
                }
            }
        }
    }

    println!();
    if config.dry_run {
        println!("{renamed} file(s) would be renamed.");
    } else {
        println!("{renamed} file(s) renamed, {errors} error(s).");
    }
}
