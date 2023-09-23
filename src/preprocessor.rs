use crate::{
    chord::Chord,
    line::{Line, LineReader, LineWriter},
    result::Result,
};
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::Path,
    str::FromStr,
};

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

    pub fn generate_markdown<P, Q>(&self, input: P, output: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let mut svgs = 0;
        let mut reader = LineReader::new(BufReader::new(File::open(input)?));
        let mut writer = LineWriter::new(BufWriter::new(File::create(output)?));

        while let Some(line) = reader.next() {
            match line {
                Line::Blank => writer.write(b"")?,
                Line::Title(line) => writer.write(titlefy(&line).as_bytes())?,
                Line::Section(line) => writer.write(sectionify(&line).as_bytes())?,
                Line::Chords(line) => {
                    let chords = chordify(&line)?;

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
                    writer.write(
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
                Line::Plain(line) => {
                    writer.write(b"````")?;

                    writer.write(line.as_bytes())?;
                    while let Some(Line::Plain(line)) = reader.peek() {
                        writer.write(line.as_bytes())?;
                        reader.next();
                    }

                    writer.write(b"````")?;
                }
            }
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
    line.split("|")
        .map(|chord| Chord::from_str(chord.trim()))
        .into_iter()
        .collect()
}
