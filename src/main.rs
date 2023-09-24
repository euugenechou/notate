use anyhow::{anyhow, Result};
use clap::Parser;
use notate::{compiler::Compiler, preprocessor::Preprocessor};
use std::{fs, path::PathBuf};

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
    #[arg(short, long)]
    preserve_artifacts: bool,

    /// Name of generated Markdown file [default: file name of tab]
    #[arg(short = 'm', long)]
    md_output: Option<String>,

    /// Name of generated PDF file [default: file name of Markdown file]
    #[arg(short = 'o', long)]
    pdf_output: Option<String>,

    /// Tab (.tab) or Markdown (.md) file to process
    #[arg()]
    input: String,
}

enum InputType {
    Tab,
    Markdown,
    Invalid,
}

impl InputType {
    fn new(input: &str) -> Self {
        if input.ends_with(".tab") {
            Self::Tab
        } else if input.ends_with(".md") {
            Self::Markdown
        } else {
            Self::Invalid
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let preprocessor = Preprocessor::new();
    let compiler = Compiler::new();

    match InputType::new(&args.input) {
        InputType::Tab => {
            let _ = fs::remove_dir_all(&args.ly_dir);
            let _ = fs::remove_dir_all(&args.svg_dir);
            fs::create_dir_all(&args.ly_dir)?;
            fs::create_dir_all(&args.svg_dir)?;

            let name = PathBuf::from(&args.input)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();

            let md_output = args.md_output.unwrap_or(format!("{name}.md"));
            let pdf_output = args.pdf_output.unwrap_or(format!("{name}.pdf"));

            preprocessor
                .set_ly_dir(&args.ly_dir)
                .set_svg_dir(&args.svg_dir)
                .generate_markdown(&args.input, &md_output)?;

            compiler.generate_pdf(&md_output, &pdf_output)?;

            if !args.preserve_artifacts {
                let _ = fs::remove_dir_all(&args.ly_dir);
                let _ = fs::remove_dir_all(&args.svg_dir);
                let _ = fs::remove_file(&md_output);
            }
        }
        InputType::Markdown => {
            let name = PathBuf::from(&args.input)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();

            let pdf_output = args.pdf_output.unwrap_or(format!("{name}.pdf"));

            compiler.generate_pdf(&args.input, &pdf_output)?;
        }
        InputType::Invalid => {
            return Err(anyhow!("input must be a tab (.tab) or Markdown (.md)"));
        }
    }

    Ok(())
}
