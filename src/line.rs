use std::{
    io::{self, BufRead, Lines, Write},
    iter::Peekable,
};

pub enum Line {
    Blank,
    Title(String),
    Section(String),
    Chords(String),
    Plain(String),
}

impl Line {
    pub fn from_line(line: String) -> Self {
        if line.is_empty() {
            Self::Blank
        } else if line.starts_with("#") {
            Self::Title(line)
        } else if line.starts_with("[") && line.ends_with("]") {
            Self::Section(line)
        } else if line.starts_with("{") && line.ends_with("}") {
            Self::Chords(line)
        } else {
            Self::Plain(line)
        }
    }
}

pub struct LineReader<R: BufRead> {
    inner: Peekable<Lines<R>>,
}

impl<R: BufRead> LineReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            inner: reader.lines().peekable(),
        }
    }

    pub fn read(&mut self) -> Option<Line> {
        self.inner.find_map(|line| line.ok()).map(Line::from_line)
    }

    pub fn peek(&mut self) -> Option<Line> {
        self.inner
            .peek()
            .iter()
            .find_map(|line| line.as_ref().ok())
            .map(|line| line.clone())
            .map(Line::from_line)
    }
}

impl<R: BufRead> Iterator for LineReader<R> {
    type Item = Line;

    fn next(&mut self) -> Option<Self::Item> {
        self.read()
    }
}

pub struct LineWriter<W> {
    inner: W,
}

impl<W: Write> LineWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }

    pub fn write(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)?;
        self.inner.write_all(b"\n")
    }
}
