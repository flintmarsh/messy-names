use std::env;
use std::fs;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use messy_names::{parse, render, FormatError, NameList};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: {} <file> [--pick CATEGORY]", args.first().map(String::as_str).unwrap_or("messy-names"));
        process::exit(2);
    }

    let path = &args[1];
    let mut pick_category: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--pick" => {
                i += 1;
                let Some(category) = args.get(i) else {
                    eprintln!("--pick requires a category name");
                    process::exit(2);
                };
                pick_category = Some(category.clone());
            }
            other => {
                eprintln!("unknown argument: {other}");
                process::exit(2);
            }
        }
        i += 1;
    }

    let source = fs::read_to_string(path).unwrap_or_else(|err| {
        eprintln!("failed to read {path}: {err}");
        process::exit(1);
    });

    match parse(&source) {
        Ok(list) => {
            print!("{}", render(&list));
            if let Some(category) = pick_category {
                pick(&list, &category);
            }
        }
        Err(errors) => {
            report_errors(&source, path, &errors);
            process::exit(1);
        }
    }
}

fn pick(list: &NameList, category: &str) {
    let Some(names) = list.categories.get(category) else {
        eprintln!("no such category: '{category}'");
        process::exit(1);
    };
    if names.is_empty() {
        eprintln!("category '{category}' has no names");
        process::exit(1);
    }

    let mut rng = Rng::new();
    let choice = &names[rng.index(names.len())];
    println!("\npicked from [{category}]: {choice}");
}

/// Print each error the way a compiler would: file:line:column, the
/// offending source line, and a caret under the exact character.
fn report_errors(source: &str, path: &str, errors: &[FormatError]) {
    let lines: Vec<&str> = source.lines().collect();
    for err in errors {
        eprintln!("{path}:{}:{}: error: {}", err.line, err.column, err.message);
        if let Some(line) = lines.get(err.line - 1) {
            eprintln!("  {line}");
            let caret_pos = err.column.saturating_sub(1);
            eprintln!("  {}^", " ".repeat(caret_pos));
        }
    }
}

/// A small, self-contained xorshift64* generator. Good enough to pick a
/// random index from a short list; not suitable for anything cryptographic.
struct Rng(u64);

impl Rng {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before the unix epoch")
            .as_nanos() as u64;
        // xorshift produces all zeroes forever if seeded with zero.
        Rng(nanos | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn index(&mut self, len: usize) -> usize {
        (self.next_u64() % len as u64) as usize
    }
}
