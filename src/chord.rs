use crate::{error::Error, result::Result};
use regex::Regex;
use std::{
    fs,
    process::{Command, Stdio},
    str::FromStr,
};

#[derive(Debug)]
pub struct Chord {
    name: String,
    left: Vec<String>,
    right: Vec<String>,
}

impl Chord {
    fn left_to_raw(&self) -> String {
        format!("<{}>", self.left.join(" "))
    }

    fn right_to_raw(&self) -> String {
        format!("<{}>", self.right.join(" "))
    }

    pub fn to_raw(&self) -> String {
        format!(
            r#"
\score {{
    <<
        \new ChordNames \with {{
            \override ChordName.font-size = #-2
        }} \chordmode {{ {} }}
        \new PianoStaff
        <<
            \new Staff = "right" \with {{
                fontSize = #-2
                \override StaffSymbol.staff-space = #(magstep -2)
                \once \override Staff.TimeSignature.stencil = ##f
            }} {{
                \clef "treble"
                \relative c'
                {{
                    {}
                }}
            }}
            \new Staff = "left" \with {{
                fontSize = #-2
                \override StaffSymbol.staff-space = #(magstep -2)
                \once \override Staff.TimeSignature.stencil = ##f
            }} {{
                \clef "bass"
                \relative c
                {{
                    {}
                }}
            }}
        >>
    >>
    \paper {{
        system-system-spacing = #'((basic-distance . 0.1) (padding . 0))
    }}
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

    pub fn ly_path<P>(&self, prefix: P) -> String
    where
        P: AsRef<str>,
    {
        format!("{}.ly", prefix.as_ref())
    }

    pub fn svg_path<P>(&self, prefix: P) -> String
    where
        P: AsRef<str>,
    {
        format!("{}.svg", prefix.as_ref())
    }

    pub fn svg_clip_path<P>(&self, prefix: P) -> String
    where
        P: AsRef<str>,
    {
        format!("{}-from-1.0.1-to-1.1.4-clip.svg", prefix.as_ref())
    }

    pub fn generate_ly<P>(&self, prefix: P) -> Result<()>
    where
        P: AsRef<str>,
    {
        fs::write(self.ly_path(prefix), self.to_raw())?;
        Ok(())
    }

    pub fn generate_svg<P>(&self, prefix: P) -> Result<()>
    where
        P: AsRef<str>,
    {
        Command::new("lilypond")
            .arg("-dbackend=svg")
            .arg("-dresolution=300")
            .arg("-dclip-systems")
            .args(["--svg", &self.ly_path(&prefix)])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap()
            .wait()?;
        Ok(())
    }
}

impl FromStr for Chord {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let re = Regex::new(r"\{(?<name>[^,]+),\s+(?<left>[^,]+),\s+(?<right>[^,]+)\}").unwrap();
        let caps = re.captures(s).ok_or(Error::MalformedChord(s.into()))?;

        Ok(Self {
            name: caps["name"].into(),
            left: caps["left"]
                .split_whitespace()
                .map(|s| s.trim().to_owned())
                .collect(),
            right: caps["right"]
                .split_whitespace()
                .map(|s| s.trim().to_owned())
                .collect(),
        })
    }
}
