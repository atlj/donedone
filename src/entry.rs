use std::{fmt::Display, path::PathBuf, usize};

#[derive(Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub line: usize,
    pub comment: Option<String>,
}

impl Entry {
    pub fn serialize(&self) -> String {
        let mut result = self.path.to_string_lossy().to_string();

        result.push('\n');
        result.push_str(&self.line.to_string());

        if let Some(comment) = &self.comment {
            result.push('\n');
            result.push_str(&comment.to_string());
        }

        result.push_str("\n\n");

        result
    }

    pub fn deserialize(input: &str) -> Result<Self, SerializeError> {
        let mut lines = input.lines();
        let path = PathBuf::from(lines.next().ok_or(SerializeError::NoPath)?);
        let line_str = lines.next().ok_or(SerializeError::NoLine)?;
        let line_number: usize = line_str.parse().ok().ok_or(SerializeError::InvalidLine)?;
        let comment = lines.next().map(str::to_string);

        Ok(Self {
            line: line_number,
            path,
            comment,
        })
    }
}

#[derive(Debug)]
pub enum SerializeError {
    NoPath,
    NoLine,
    InvalidLine,
}

impl Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPath => write!(f, "No path found in file"),
            Self::NoLine => write!(f, "No line number found in file"),
            Self::InvalidLine => write!(f, "Found non-integer line"),
        }
    }
}
