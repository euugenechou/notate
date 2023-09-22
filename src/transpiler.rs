use crate::{chord::Chord, error::Error, result::Result};
use path_macro::path;
use std::{
    fs,
    io::{self, BufRead, Write},
    iter,
    str::FromStr,
};

struct LineWriter<W> {
    inner: W,
}

impl<W: Write> LineWriter<W> {
    fn new(inner: W) -> Self {
        Self { inner }
    }

    fn writeln(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)?;
        self.inner.write_all(b"\n")
    }
}

impl<W: Write> Write for LineWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

pub struct Transpiler;

impl Transpiler {
    pub fn generate_pdf<R, W>(reader: R, writer: W) -> Result<()>
    where
        R: BufRead,
        W: Write,
    {
        let mut pngs = 0;
        let pngdir = "pngs";

        // Recreate the PNG directory.
        let _ = fs::remove_dir_all(pngdir);
        fs::create_dir_all(pngdir)?;

        let mut reader = reader
            .lines()
            .filter_map(|line| line.ok())
            .map(|line| line.to_owned());
        let mut writer = LineWriter::new(writer);

        while let Some(line) = reader.next() {
            // Blank line.
            if line.is_empty() {
                writer.writeln(b"")?;
                continue;
            }

            // Title line.
            if line.starts_with("#") {
                writer.writeln(titlefy(&line).as_bytes())?;
                continue;
            }

            // Line is some sort of section heading.
            if line.starts_with("[") && line.ends_with("]") {
                writer.writeln(sectionify(&line).as_bytes())?;
                continue;
            }

            // Purely a naming decision (pun intended).
            let names = line;
            let lyrics = reader.next().ok_or(Error::MalformedTab)?;
            let chords = chordify(&reader.next().ok_or(Error::MalformedTab)?)?;

            // Emit a code block with the chord names and lyrics.
            writer.writeln(b"```")?;
            writer.writeln(names.as_bytes())?;
            writer.writeln(lyrics.as_bytes())?;
            writer.writeln(b"```")?;

            // Paths to the chord PNGs.
            let paths = (0..chords.len())
                .map(|n| path![pngdir / format!("{}.png", pngs + n)])
                .collect::<Vec<_>>();

            // Generate the PNGs.
            for (chord, path) in iter::zip(chords.iter(), paths.iter()) {
                chord.generate_png(path)?;
            }

            // Emit the PNGs inline.
            writer.writeln(
                paths
                    .iter()
                    .map(|path| format!("![]({})", path.to_str().unwrap()))
                    .collect::<Vec<_>>()
                    .join(" &nbsp; &nbsp; ")
                    .as_bytes(),
            )?;

            pngs += paths.len();
        }

        Ok(())
    }
}

fn titlefy(line: &str) -> String {
    format!(
        "# `{}`",
        line.split_once(" ")
            .map(|(_, title)| title.to_owned())
            .unwrap()
    )
}

fn sectionify(line: &str) -> String {
    format!("### `{}`", line.replace("[", "").replace("]", ""))
}

fn chordify(line: &str) -> Result<Vec<Chord>> {
    line.split("|")
        .map(|chord| Chord::from_str(chord.trim()))
        .into_iter()
        .collect()
}
