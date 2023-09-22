use crate::{chord::Chord, error::Error, result::Result};
use std::{
    fs,
    io::{self, BufRead, Write},
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

pub struct Transpiler {
    ly_dir: String,
    svg_dir: String,
}

impl Transpiler {
    pub fn new<P, Q>(ly_dir: P, svg_dir: Q) -> Self
    where
        P: AsRef<str>,
        Q: AsRef<str>,
    {
        Self {
            ly_dir: ly_dir.as_ref().into(),
            svg_dir: svg_dir.as_ref().into(),
        }
    }

    pub fn reset(&self) -> Result<()> {
        let _ = fs::remove_dir_all(&self.ly_dir);
        let _ = fs::remove_dir_all(&self.svg_dir);
        fs::create_dir_all(&self.ly_dir)?;
        fs::create_dir_all(&self.svg_dir)?;
        Ok(())
    }

    pub fn generate_markdown<R, W>(&self, reader: R, writer: W) -> Result<()>
    where
        R: BufRead,
        W: Write,
    {
        let mut svgs = 0;

        let mut reader = reader
            .lines()
            .filter_map(|line| line.ok())
            .map(|line| line.to_owned());
        let mut writer = LineWriter::new(writer);

        self.reset()?;

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

            // Generate the Lilypond source and SVGs.
            for (i, chord) in chords.iter().enumerate() {
                let prefix = (svgs + i).to_string();

                chord.generate_ly(&prefix)?;
                chord.generate_svg(&prefix)?;

                fs::rename(
                    chord.ly_path(&prefix),
                    format!("{}/{}", self.ly_dir, chord.ly_path(&prefix)),
                )?;
                fs::remove_file(chord.svg_path(&prefix))?;
                fs::rename(
                    chord.svg_clip_path(&prefix),
                    format!("{}/{}", self.svg_dir, chord.svg_path(&prefix)),
                )?;
            }

            // Emit the SVGs inline.
            writer.writeln(
                chords
                    .iter()
                    .enumerate()
                    .map(|(i, chord)| {
                        let path = (svgs + i).to_string();
                        format!("![]({}/{})", self.svg_dir, chord.svg_path(path))
                    })
                    .collect::<Vec<_>>()
                    .join(" &nbsp; &nbsp; ")
                    .as_bytes(),
            )?;

            svgs += chords.len();
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
