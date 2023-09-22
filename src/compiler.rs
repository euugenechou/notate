use crate::result::Result;
use pandoc::{OutputKind, Pandoc};

pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate_pdf<P, Q>(self, input: P, output: Q) -> Result<()>
    where
        P: AsRef<str>,
        Q: AsRef<str>,
    {
        let mut pandoc = Pandoc::new();

        pandoc
            .add_input(input.as_ref())
            .set_output(OutputKind::File(output.as_ref().into()));

        pandoc.execute().unwrap();

        Ok(())
    }
}
