use anyhow::Result;
use clap::Parser;
use notate::transpiler::Transpiler;
use std::{
    fs,
    io::{stdin, stdout, BufReader},
};

/// Transpiles basic guitar tabs annotated with piano chords to Markdown.
#[derive(Parser)]
struct Args {
    /// Directory for generated Lilypond source files
    #[arg(short, long, default_value = "lys")]
    ly_dir: String,

    /// Directory for generated chord SVG files
    #[arg(short, long, default_value = "svgs")]
    svg_dir: String,

    /// Remove generated artifacts
    #[arg(short, long, default_value_t = false)]
    remove_artifacts: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let transpiler = Transpiler::new(&args.ly_dir, &args.svg_dir);

    transpiler.generate_markdown(BufReader::new(stdin()), stdout())?;

    if args.remove_artifacts {
        let _ = fs::remove_dir_all(&args.ly_dir);
        let _ = fs::remove_dir_all(&args.svg_dir);
    }

    Ok(())
}
