use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    Auto,
    Dbml,
    WaveDrom,
    D2,
    Structurizr,
    LikeC4,
    Nomnoml,
    Pikchr,
}

impl Format {
    pub fn detect(source: &str) -> Self {
        let trimmed = source.trim_start();
        let lower = trimmed.to_ascii_lowercase();

        if trimmed.starts_with('{')
            && (lower.contains("signal:")
                || lower.contains("\"signal\"")
                || lower.contains("reg:")
                || lower.contains("\"reg\""))
        {
            return Self::WaveDrom;
        }

        if lower.contains("specification") && (lower.contains("model") || lower.contains("views")) {
            return Self::LikeC4;
        }

        if lower.contains("workspace")
            && (lower.contains("softwaresystem")
                || lower.contains("systemcontext")
                || lower.contains("container"))
        {
            return Self::Structurizr;
        }

        if lower.contains("table ")
            || lower.starts_with("table ")
            || lower.contains("ref:")
            || lower.contains("enum ")
        {
            return Self::Dbml;
        }

        if lower.lines().any(|line| {
            let line = line.trim_start();
            line.starts_with('#') || line.starts_with('[')
        }) && lower.contains('[')
            && lower.contains(']')
        {
            return Self::Nomnoml;
        }

        if lower.lines().any(|line| {
            matches!(
                line.split_whitespace().next(),
                Some(
                    "box"
                        | "circle"
                        | "ellipse"
                        | "arrow"
                        | "line"
                        | "arc"
                        | "spline"
                        | "cylinder"
                        | "diamond"
                        | "oval"
                        | "move"
                )
            )
        }) {
            return Self::Pikchr;
        }

        Self::D2
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Dbml => "dbml",
            Self::WaveDrom => "wavedrom",
            Self::D2 => "d2",
            Self::Structurizr => "structurizr",
            Self::LikeC4 => "likec4",
            Self::Nomnoml => "nomnoml",
            Self::Pikchr => "pikchr",
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Format {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "dbml" => Ok(Self::Dbml),
            "wavedrom" | "wavejson" => Ok(Self::WaveDrom),
            "d2" => Ok(Self::D2),
            "structurizr" | "structurizr-dsl" => Ok(Self::Structurizr),
            "likec4" | "c4" => Ok(Self::LikeC4),
            "nomnoml" => Ok(Self::Nomnoml),
            "pikchr" | "pic" => Ok(Self::Pikchr),
            other => Err(format!("unsupported format: {other}")),
        }
    }
}
