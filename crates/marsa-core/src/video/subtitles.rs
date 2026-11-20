//! Subtitles Marsa asks yt-dlp to write beside a video. Only fixed language
//! codes and two fixed formats are allowed: nothing from a page can become
//! an option of the tool (ADR-0009).

use super::VideoError;

/// The format of the subtitle files yt-dlp writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubFormat {
    Vtt,
    Srt,
}

impl SubFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            SubFormat::Vtt => "vtt",
            SubFormat::Srt => "srt",
        }
    }
}

/// Which subtitles to save: `langs` is a comma-separated list of language
/// codes (e.g. `ar,en`), or empty for every language the site offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subtitles {
    pub langs: String,
    pub format: SubFormat,
}

impl Subtitles {
    /// Checks and normalises the user's choice: at most 8 codes of 2 to 12
    /// letters, digits or dashes, no spaces, and one of the two formats.
    pub fn new(langs: &str, format: &str) -> Result<Self, VideoError> {
        let format = match format.trim().to_ascii_lowercase().as_str() {
            "vtt" => SubFormat::Vtt,
            "srt" => SubFormat::Srt,
            _ => {
                return Err(VideoError::Refused(format!(
                    "an unknown subtitle format: {format:?}"
                )));
            }
        };
        let langs = langs.trim();
        let normalised = if langs.is_empty() {
            String::new()
        } else {
            let parts: Vec<&str> = langs.split(',').map(str::trim).collect();
            if parts.len() > 8
                || parts.iter().any(|p| {
                    p.is_empty()
                        || p.len() > 12
                        || !p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                })
            {
                return Err(VideoError::Refused(format!(
                    "invalid subtitle languages: {langs:?}"
                )));
            }
            parts.join(",")
        };
        Ok(Self {
            langs: normalised,
            format,
        })
    }

    /// `all` when no language was specified, else the comma-separated list.
    pub fn langs_arg(&self) -> &str {
        if self.langs.is_empty() {
            "all"
        } else {
            &self.langs
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalises_languages_and_format() {
        let subs = Subtitles::new(" ar , en ", "VTT").unwrap();
        assert_eq!(subs.langs, "ar,en");
        assert_eq!(subs.format, SubFormat::Vtt);
        assert_eq!(Subtitles::new("", "srt").unwrap().langs_arg(), "all");
    }

    #[test]
    fn refuses_injection_and_unknown_formats() {
        assert!(Subtitles::new("en --exec x", "vtt").is_err());
        assert!(Subtitles::new("en", "mp3").is_err());
        assert!(Subtitles::new("../../etc", "vtt").is_err());
        assert!(Subtitles::new(&",".repeat(9), "vtt").is_err());
    }
}
