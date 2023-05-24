use anyhow::{Context, Result};
use log::{info, warn};
use std::io::{self, Write};

use clap::Parser;

fn main() -> Result<()> {
    println!("Hello, world!");
    let args = encsync::Cli::parse();
    env_logger::init();
    info!("starting up");
    warn!("oops, nothing implemented!");
    let stdout = io::stdout(); // get the global stdout entity
    let mut handle = stdout.lock(); // acquire a lock on it
    writeln!(handle, "foo: {}", 42)?; // add `?` if you care about errors here

    // let content = std::fs::read_to_string(&args.path).expect("could not read file");

    let content = std::fs::read_to_string(&args.path)
        .with_context(|| format!("could not read file `{}`", &args.path.display()))?;
    println!("file content: {}", content);

    let pb = indicatif::ProgressBar::new(100);
    for i in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        pb.println(format!("[+] finished #{}", i));
        pb.inc(1);
    }
    pb.finish_with_message("done");


    encsync::find_matches(&content, &args.pattern, &mut std::io::stdout());

    Ok(())
}

// fn answer() -> i32 {
//     42
// }

// #[test]
// fn check_answer_validity() {
//     assert_eq!(answer(), 42);
// }


#[test]
fn find_a_match() {
    let mut result = Vec::new();
    encsync::find_matches("lorem ipsum\ndolor sit amet", "lorem", &mut result);
    assert_eq!(result, b"lorem ipsum\n");
}