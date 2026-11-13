//! FTP / FTPS downloads (RFC 3659): probe (`SIZE`), resume (`REST`),
//! passive transfer (`PASV`). Plain FTP and implicit FTPS (port 990).

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use reqwest::Url;

use super::Error;

/// A probed FTP file: its name and size, if the server said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FtpProbe {
    pub name: String,
    pub size: Option<u64>,
}

/// What [`ftp_download`] wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FtpFinished {
    pub bytes: u64,
}

fn parse_ftp_url(raw: &str) -> Result<Url, Error> {
    let url = Url::parse(raw).map_err(|e| Error::InvalidUrl {
        url: raw.to_owned(),
        reason: e.to_string(),
    })?;
    match url.scheme() {
        "ftp" | "ftps" => Ok(url),
        other => Err(Error::UnsupportedScheme(other.to_owned())),
    }
}

/// Any byte stream behind the FTP dialect.
trait FtpStream: Read + Write {}
impl FtpStream for TcpStream {}
impl FtpStream for rustls::StreamOwned<rustls::ClientConnection, TcpStream> {}

struct Ftp {
    control: BufReader<Box<dyn FtpStream + Send>>,
}

impl Ftp {
    fn connect(url: &Url) -> Result<Ftp, Error> {
        let host = url
            .host_str()
            .ok_or_else(|| Error::Ftp("FTP URL has no host".into()))?;
        let is_tls = url.scheme() == "ftps";
        let port = url
            .port_or_known_default()
            .unwrap_or(if is_tls { 990 } else { 21 });
        let tcp = TcpStream::connect((host.to_owned(), port))
            .map_err(|e| Error::Ftp(format!("FTP connect {host}:{port}: {e}")))?;
        tcp.set_read_timeout(Some(Duration::from_secs(30))).ok();
        let stream: Box<dyn FtpStream + Send> = if is_tls {
            let native = rustls_native_certs::load_native_certs();
            let mut roots = rustls::RootCertStore::empty();
            for der in native.certs {
                roots.add(der).ok();
            }
            let config = rustls::ClientConfig::builder()
                .with_root_certificates(roots)
                .with_no_client_auth();
            let cfg = Arc::new(config);
            let server_name = rustls::pki_types::ServerName::try_from(host.to_owned())
                .map_err(|e| Error::Ftp(format!("bad server name: {e}")))?;
            let connector = rustls::ClientConnection::new(cfg, server_name)
                .map_err(|e| Error::Ftp(format!("TLS handshake prepare: {e}")))?;
            // `StreamOwned` handshakes on first I/O; errors surface on read.
            Box::new(rustls::StreamOwned::new(connector, tcp)) as _
        } else {
            Box::new(tcp) as _
        };
        let mut session = Ftp {
            control: BufReader::new(stream),
        };
        session.read_reply()?;
        let (user, pass) = match (url.username(), url.password()) {
            ("", _) => ("anonymous".to_owned(), "marsa@".to_owned()),
            (u, Some(p)) => (u.to_owned(), p.to_owned()),
            (u, None) => (u.to_owned(), String::new()),
        };
        session.command(&format!("USER {user}"))?;
        if !pass.is_empty() {
            session.command(&format!("PASS {pass}"))?;
        }
        session.command("TYPE I")?;
        Ok(session)
    }

    fn read_reply(&mut self) -> Result<(u16, String), Error> {
        let mut text = String::new();
        loop {
            let mut line = String::new();
            self.control
                .read_line(&mut line)
                .map_err(|e| Error::Ftp(format!("FTP reply: {e}")))?;
            text.push_str(line.trim_end());
            let rest = line.trim_start();
            if rest.len() < 3 || !rest[..3].chars().all(|c| c.is_ascii_digit()) {
                return Err(Error::Ftp(format!("FTP reply not numeric: {rest:?}")));
            }
            let code = rest[..3]
                .parse::<u16>()
                .map_err(|_| Error::Ftp(format!("FTP code parse: {rest:?}")))?;
            if rest.chars().nth(3) != Some('-') {
                return Ok((code, text.trim().to_owned()));
            }
        }
    }

    fn command(&mut self, cmd: &str) -> Result<(u16, String), Error> {
        self.control
            .get_mut()
            .write_all(format!("{cmd}\r\n").as_bytes())
            .map_err(|e| Error::Ftp(format!("FTP send {cmd:?}: {e}")))?;
        let (code, reply) = self.read_reply()?;
        if (400..600).contains(&code) {
            return Err(Error::Ftp(format!("FTP {cmd:?} answered {code}: {reply}")));
        }
        Ok((code, reply))
    }

    /// The passive data address: the six numbers in the PASV reply.
    fn open_data(&mut self) -> Result<TcpStream, Error> {
        let (_, reply) = self.command("PASV")?;
        let nums: Vec<u8> = reply
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse().ok())
            .collect();
        if nums.len() < 6 {
            return Err(Error::Ftp(format!("PASV reply: {reply:?}")));
        }
        let host = format!("{}.{}.{}.{}", nums[0], nums[1], nums[2], nums[3]);
        let port = (u16::from(nums[4]) << 8) | u16::from(nums[5]);
        let stream = TcpStream::connect((host.as_str(), port))
            .map_err(|e| Error::Ftp(format!("FTP data {host}:{port}: {e}")))?;
        stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
        Ok(stream)
    }
}

fn first_number(reply: &str) -> Option<u64> {
    reply.split_whitespace().find_map(|w| w.parse::<u64>().ok())
}

/// Probe an FTP file: its name and size (`SIZE`), without downloading.
pub fn ftp_probe(raw: &str) -> Result<FtpProbe, Error> {
    let url = parse_ftp_url(raw)?;
    let path = url.path().trim_start_matches('/');
    let name = path.rsplit('/').next().unwrap_or("download").to_owned();
    let mut session = Ftp::connect(&url)?;
    let size = session
        .command(&format!("SIZE {path}"))
        .ok()
        .and_then(|(_, reply)| first_number(&reply));
    let _ = session.command("QUIT");
    Ok(FtpProbe { name, size })
}

/// Download an FTP file to `dest`, resuming from `resume_from` with `REST`
/// when the server allows it.
pub fn ftp_download(
    raw: &str,
    dest: &std::path::Path,
    resume_from: u64,
) -> Result<FtpFinished, Error> {
    let url = parse_ftp_url(raw)?;
    let path = url.path().trim_start_matches('/');
    let mut session = Ftp::connect(&url)?;
    if resume_from > 0 {
        session.command(&format!("REST {resume_from}"))?;
    }
    let mut data = session.open_data()?;
    session.command(&format!("RETR {path}"))?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(resume_from > 0)
        .write(true)
        .open(dest)
        .map_err(|e| Error::Ftp(format!("write {}: {e}", dest.display())))?;
    let mut written: u64 = resume_from;
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = data
            .read(&mut buf)
            .map_err(|e| Error::Ftp(format!("FTP data: {e}")))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])
            .map_err(|e| Error::Ftp(format!("write {}: {e}", dest.display())))?;
        written += n as u64;
    }
    let _ = session.command("QUIT");
    Ok(FtpFinished { bytes: written })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ftp_url_rejects_http() {
        assert!(parse_ftp_url("http://example.com/f").is_err());
    }

    #[test]
    fn ftp_url_needs_a_host() {
        // A control-plane client is never handed an endpoint without one;
        // the URL parser itself refuses an empty host for the FTP schemes.
        assert!(parse_ftp_url("ftp:").is_err());
    }

    #[test]
    fn ftps_url_parses() {
        assert!(parse_ftp_url("ftps://ftp.example.com/f").is_ok());
    }
}
