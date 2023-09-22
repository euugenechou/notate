use anyhow::{anyhow, Result};
use clap::Parser;
use notate::{compiler::Compiler, preprocessor::Preprocessor};
use std::fs;

/// Compiles basic guitar tabs annotated with piano chords to PDF.
#[derive(Parser)]
struct Args {
    /// Directory for generated Lilypond source files
    #[arg(short, long, default_value = "lys")]
    ly_dir: String,

    /// Directory for generated chord SVG files
    #[arg(short, long, default_value = "svgs")]
    svg_dir: String,

    /// Preserve generated intermediate artifacts
    #[arg(short, long, default_value_t = false)]
    preserve_artifacts: bool,

    /// Name of generated Markdown file
    #[arg(short = 'm', long, default_value = "a.md")]
    md_output: String,

    /// Name of generated PDF
    #[arg(short = 'o', long, default_value = "a.pdf")]
    pdf_output: String,

    /// Tab (.tab) or Markdown (.md) file to process
    #[arg()]
    input: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let preprocessor = Preprocessor::new();
    let compiler = Compiler::new();

    if args.input.ends_with(".tab") {
        preprocessor
            .set_ly_dir(&args.ly_dir)
            .set_svg_dir(&args.svg_dir)
            .generate_markdown(&args.input, &args.md_output)?;
        compiler.generate_pdf(&args.md_output, &args.pdf_output)?;
    } else if args.input.ends_with(".md") {
        compiler.generate_pdf(&args.input, &args.pdf_output)?;
    } else {
        return Err(anyhow!("input must be a tab (.tab) or Markdown (.md)"));
    }

    if !args.preserve_artifacts {
        let _ = fs::remove_dir_all(&args.ly_dir);
        let _ = fs::remove_dir_all(&args.svg_dir);
        let _ = fs::remove_file(&args.md_output);
    }

    Ok(())
}
