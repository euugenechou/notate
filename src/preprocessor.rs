use crate::{chord::Chord, error::Error, result::Result};
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::Path,
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

pub struct Preprocessor {
    ly_dir: String,
    svg_dir: String,
}

impl Preprocessor {
    pub fn new() -> Self {
        Self {
            ly_dir: "lys".into(),
            svg_dir: "svgs".into(),
        }
    }

    pub fn set_ly_dir<P: AsRef<str>>(mut self, path: P) -> Self {
        self.ly_dir = path.as_ref().into();
        self
    }

    pub fn set_svg_dir<P: AsRef<str>>(mut self, path: P) -> Self {
        self.svg_dir = path.as_ref().into();
        self
    }

    pub fn reset(&self) -> Result<()> {
        let _ = fs::remove_dir_all(&self.ly_dir);
        let _ = fs::remove_dir_all(&self.svg_dir);
        fs::create_dir_all(&self.ly_dir)?;
        fs::create_dir_all(&self.svg_dir)?;
        Ok(())
    }

    pub fn generate_markdown<P, Q>(&self, input: P, output: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let mut svgs = 0;

        let mut reader = BufReader::new(File::open(input)?)
            .lines()
            .filter_map(|line| line.ok())
            .map(|line| line.to_owned());

        let mut writer = LineWriter::new(BufWriter::new(File::create(output)?));

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
        "# `# {}`",
        line.split_once(" ")
            .map(|(_, title)| title.to_owned())
            .unwrap()
    )
}

fn sectionify(line: &str) -> String {
    format!("### `## {}`", line.replace("[", "").replace("]", ""))
}

fn chordify(line: &str) -> Result<Vec<Chord>> {
    if line.contains("skip") {
        return Ok(vec![]);
    }

    line.split("|")
        .map(|chord| Chord::from_str(chord.trim()))
        .into_iter()
        .collect()
}
