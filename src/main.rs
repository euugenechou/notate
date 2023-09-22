use anyhow::Result;
use notation::transpiler::Transpiler;
use std::io::{stdin, stdout, BufReader};

fn main() -> Result<()> {
    Transpiler::generate_pdf(BufReader::new(stdin()), stdout())?;
    Ok(())
}
