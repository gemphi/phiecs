//! The Quest runner: a persona file's plan, asked of an oracle, written to
//! disk as the rows every reader counts (docs/159 step 4).
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c), grows from it. A persona (`phiecs::persona::Persona`,
//! a markdown file) names its seeds, the kinds it asks, its voice and how
//! far to walk; this runs one `Quest::for_persona` per seed and writes
//! `<root>/<slug>/<seed>/{README.md, steps/, relations.tsv, sentences.tsv}`,
//! which `phient::relate::read_quests` and `bin/persona_from_quest` read.
//!
//! ```text
//! cargo run --release -p phiecs --example persona_quest -- <root> <persona.md | dir>… \
//!     [--model phi4:latest] [--oracle <program> [--arg <a>]…] [--only <seed>] [--dry]
//! ```
//!
//! A directory means every `*.md` in it but `README.md`, so on PowerShell
//! (which does not expand `*.md`) pass the folder: `quests crates\phinet\personas`.
//!
//! The oracle is `ollama run <model>` unless `--oracle` names another
//! program that answers a prompt on stdin with text on stdout. `--dry`
//! prints the plan and writes nothing. A seed folder that already holds a
//! `relations.tsv` is skipped, so a run can be resumed.
//!
//! Without ollama, `examples/oracle.py` is a stand-in that answers in the
//! one-kind format from a fixed pool, enough to exercise the whole chain:
//!
//! ```text
//! cargo run -p phiecs --example persona_quest -- quests crates/phiecs/examples/sailor.md \
//!     --oracle python3 --arg -I --arg crates/phiecs/examples/oracle.py
//! cargo run --release --bin persona_from_quest -- quests crates/phiecs/examples/sailor.md
//! ```

use phiecs::persona::Persona;
use phiecs::quest::{Oracle, Quest, Spawned};
use std::path::{Path, PathBuf};

fn flag(a: &mut Vec<String>, name: &str) -> Option<String> {
    let i = a.iter().position(|x| x == name)?;
    let v = a.get(i + 1).cloned();
    a.drain(i..(i + 2).min(a.len()));
    v
}

fn main() {
    let mut a: Vec<String> = std::env::args().skip(1).collect();
    let model = flag(&mut a, "--model").unwrap_or_else(|| "phi4:latest".into());
    let program = flag(&mut a, "--oracle");
    let mut args = Vec::new();
    while let Some(x) = flag(&mut a, "--arg") {
        args.push(x);
    }
    let only = flag(&mut a, "--only");
    let dry = a.iter().position(|x| x == "--dry").map(|i| a.remove(i)).is_some();
    let Some(root) = a.first().map(PathBuf::from) else {
        eprintln!("usage: persona_quest <root> <persona.md>… [--model m] [--oracle prog [--arg a]…] [--only seed] [--dry]");
        std::process::exit(2);
    };
    if a.len() < 2 {
        eprintln!("no persona file given");
        std::process::exit(2);
    }
    let mut oracle: Box<dyn Oracle> = match &program {
        Some(p) => Box::new(Spawned::new(p, &args.iter().map(String::as_str).collect::<Vec<_>>())),
        None => Box::new(Spawned::ollama(&model)),
    };
    for file in a[1..].iter().flat_map(|x| persona_files(x)) {
        let file = file.to_string_lossy().to_string();
        let text = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("cannot read {file}: {e}"));
        let p = Persona::from_md(&text).unwrap_or_else(|e| panic!("{file}: {e}"));
        println!("{} ({}): {} seeds × {} kinds, {} questions per seed, depth {}", p.name, p.slug(), p.seeds.len(), p.kinds.len(), p.per_seed, p.depth);
        for seed in p.seeds.iter().filter(|s| only.as_ref().is_none_or(|o| o == *s)) {
            run_seed(&root.join(p.slug()), &p, seed, oracle.as_mut(), dry);
        }
    }
}

/// A persona file as given; a directory's `*.md` files (README aside); or
/// a `dir\*.md` pattern a shell such as PowerShell did not expand.
fn persona_files(arg: &str) -> Vec<PathBuf> {
    let arg = arg.trim_end_matches(['/', '\\']);
    let dir = match arg.rsplit_once(['/', '\\']) {
        Some((d, pat)) if pat.contains('*') => PathBuf::from(d),
        _ if arg.contains('*') => PathBuf::from("."),
        _ => {
            let p = PathBuf::from(arg);
            return match p.is_dir() {
                true => persona_files(&format!("{arg}/*.md")),
                false => vec![p],
            };
        }
    };
    let mut v: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|rd| rd.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default();
    v.retain(|p| p.extension().is_some_and(|x| x == "md") && p.file_name().is_some_and(|n| n != "README.md"));
    v.sort();
    v
}

fn run_seed(dir: &Path, p: &Persona, seed: &str, oracle: &mut dyn Oracle, dry: bool) {
    let out = dir.join(seed);
    if out.join("relations.tsv").exists() {
        println!("  {seed}: already run ({}), skipped", out.display());
        return;
    }
    if dry {
        println!("  {seed}: would ask {} and write {}", oracle.name(), out.display());
        return;
    }
    let mut quest = Quest::for_persona(p, seed);
    let asked = quest.run_watching(oracle, |q, step| {
        println!("  {seed} · {} · {} kept, {} opened, {} pending", step.about, step.kept.len(), step.opened.len(), q.pending_questions());
    });
    if let Some(why) = &quest.stopped {
        eprintln!("  {seed}: stopped — {why}");
    }
    match quest.write(&out, &oracle.name()) {
        Ok(()) => println!(
            "  {seed}: {asked} questions → {} words, {} kinds, {} edges, {} sentences → {}",
            quest.world.lexicon.len(),
            quest.world.relations.len(),
            quest.world.edges.len(),
            quest.sentences.len(),
            out.display()
        ),
        Err(e) => eprintln!("  {seed}: cannot write {}: {e}", out.display()),
    }
}
