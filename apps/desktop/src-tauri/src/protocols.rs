//! Protocol Manager (v1): one place that answers "which engine handles this
//! link?". Adding a future protocol (e.g. a plugin protocol) touches only
//! this file and the corresponding engine branch.
//!
//! Today: `http`/`https` → direct engine, `ftp`/`ftps` → `engine/ftp`,
//! `magnet` → torrent engine (`librqbit`).

/// The download protocol a link belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolKind {
    Http,
    Ftp,
    Torrent,
}

impl ProtocolKind {
    /// Which engine a URL uses, by its scheme.
    pub fn of(url: &str) -> Option<ProtocolKind> {
        let trimmed = url.trim();
        let scheme = trimmed.split(':').next()?.to_ascii_lowercase();
        match scheme.as_str() {
            "http" | "https" => Some(ProtocolKind::Http),
            "ftp" | "ftps" => Some(ProtocolKind::Ftp),
            "magnet" => Some(ProtocolKind::Torrent),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocols_by_scheme() {
        assert_eq!(ProtocolKind::of("https://x/f"), Some(ProtocolKind::Http));
        assert_eq!(ProtocolKind::of("ftp://x/f"), Some(ProtocolKind::Ftp));
        assert_eq!(ProtocolKind::of("ftps://x/f"), Some(ProtocolKind::Ftp));
        assert_eq!(
            ProtocolKind::of("magnet:?xt=urn:btih:x"),
            Some(ProtocolKind::Torrent)
        );
        assert_eq!(ProtocolKind::of("ssh://x"), None);
    }
}
