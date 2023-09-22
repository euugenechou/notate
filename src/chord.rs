use crate::{error::Error, result::Result};
use regex::Regex;
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    str::FromStr,
};

#[derive(Debug)]
pub struct Chord {
    name: String,
    right: Vec<String>,
    left: Vec<String>,
}

impl Chord {
    fn right_to_raw(&self) -> String {
        format!("<{}>", self.right.join(" "))
    }

    fn left_to_raw(&self) -> String {
        format!("<{}>", self.left.join(" "))
    }

    pub fn to_raw(&self) -> String {
        format!(
            r#"
\score {{
    <<
        \new ChordNames \chordmode {{ {} }}
        \new PianoStaff
        <<
            \new Staff = "right" {{
                \clef "treble"
                \relative c'
                {{
                    \once \override Staff.TimeSignature.stencil = ##f
                    {}
                }}
            }}
            \new Staff = "left" {{
                \clef "bass"
                \relative c
                {{
                    \once \override Staff.TimeSignature.stencil = ##f
                    {}
                }}
            }}
        >>
    >>
    \layout {{
        clip-regions = #(list
            (cons
                (make-rhythmic-location 1 0 4)
                (make-rhythmic-location 1 1 4)
            )
        )
    }}
}}
    "#,
            &self.name,
            &self.right_to_raw(),
            &self.left_to_raw()
        )
    }

    pub fn generate_ly<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        fs::write(path, self.to_raw())?;
        Ok(())
    }

    pub fn generate_png<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        self.generate_ly("tmp.ly")?;

        Command::new("lilypond")
            .arg("-dbackend=eps")
            .arg("-dresolution=1200")
            .arg("-dclip-systems")
            .args(["--png", "tmp.ly"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        fs::remove_file("tmp.ly")?;
        fs::remove_file("tmp.png")?;
        fs::rename("tmp-from-1.0.1-to-1.1.4-clip.png", path)?;

        Ok(())
    }
}

impl FromStr for Chord {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let re = Regex::new(r"\{(?<name>[^,]+),\s+(?<right>[^,]+),\s+(?<left>[^,]+)\}").unwrap();
        let caps = re.captures(s).ok_or(Error::MalformedChord(s.into()))?;

        Ok(Self {
            name: caps["name"].into(),
            right: caps["right"]
                .split_whitespace()
                .map(|s| s.trim().to_owned())
                .collect(),
            left: caps["right"]
                .split_whitespace()
                .map(|s| s.trim().to_owned())
                .collect(),
        })
    }
}
