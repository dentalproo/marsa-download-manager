//! Torrent core (v1 foundation): bencode + `.torrent` + Magnet parsing.
//! Peer download (librqbit) comes later; this is the analysis layer:
//! name, files, total size, pieces — used by Torrent Health Analyzer and
//! Security Center before adding.

use std::collections::BTreeMap;

pub type Dict = BTreeMap<String, Value>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Dict(Dict),
}

impl Value {
    pub fn int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }
    pub fn bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(b) => Some(b),
            _ => None,
        }
    }
    pub fn str(&self) -> Option<String> {
        self.bytes()
            .map(|b| String::from_utf8_lossy(b).into_owned())
    }
    pub fn dict(&self) -> Option<&Dict> {
        match self {
            Value::Dict(d) => Some(d),
            _ => None,
        }
    }
    pub fn list(&self) -> Option<&[Value]> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }
}

/// Decode bencode from `input` starting at `pos`; returns value + next pos.
pub fn decode(input: &[u8], pos: usize) -> Result<(Value, usize), String> {
    match input.get(pos) {
        Some(b'i') => {
            let end = input[pos..]
                .iter()
                .position(|&c| c == b'e')
                .ok_or_else(|| "int without e".to_owned())?;
            let text =
                std::str::from_utf8(&input[pos + 1..pos + end]).map_err(|e| e.to_string())?;
            Ok((
                Value::Int(text.parse::<i64>().map_err(|e| e.to_string())?),
                pos + end + 1,
            ))
        }
        Some(b'l') => {
            let mut items = Vec::new();
            let mut cur = pos + 1;
            loop {
                if input.get(cur) == Some(&b'e') {
                    return Ok((Value::List(items), cur + 1));
                }
                let (value, next) = decode(input, cur)?;
                items.push(value);
                cur = next;
            }
        }
        Some(b'd') => {
            let mut dict = BTreeMap::new();
            let mut cur = pos + 1;
            loop {
                if input.get(cur) == Some(&b'e') {
                    return Ok((Value::Dict(dict), cur + 1));
                }
                let (key, next) = decode(input, cur)?;
                let key = key
                    .str()
                    .ok_or_else(|| "dict key is not bytes".to_owned())?;
                let (value, next2) = decode(input, next)?;
                dict.insert(key, value);
                cur = next2;
            }
        }
        Some(c) if c.is_ascii_digit() => {
            let colon = input[pos..]
                .iter()
                .position(|&c| c == b':')
                .ok_or_else(|| "string without colon".to_owned())?;
            let len: usize = std::str::from_utf8(&input[pos..pos + colon])
                .map_err(|e| e.to_string())?
                .parse()
                .map_err(|e: std::num::ParseIntError| e.to_string())?;
            let start = pos + colon + 1;
            let end = start
                .checked_add(len)
                .ok_or_else(|| "string length overflow".to_owned())?;
            if end > input.len() {
                return Err("string overruns input".into());
            }
            Ok((Value::Bytes(input[start..end].to_vec()), end))
        }
        _ => Err(format!("bad bencode byte at {pos}")),
    }
}

/// One file inside a multi-file torrent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TorrentFile {
    pub path: String,
    pub length: u64,
}

/// What a `.torrent` announces before any peer is contacted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TorrentInfo {
    pub name: String,
    pub files: Vec<TorrentFile>,
    pub total: u64,
    pub piece_length: u64,
    pub piece_count: u64,
}

/// Parse a `.torrent` file's bencode into [`TorrentInfo`].
pub fn parse_torrent(data: &[u8]) -> Result<TorrentInfo, String> {
    let (top, _) = decode(data, 0)?;
    let top = top
        .dict()
        .ok_or_else(|| "torrent top is not a dict".to_owned())?;
    let info = top
        .get("info")
        .and_then(|v| v.dict())
        .ok_or_else(|| "no info dict".to_owned())?;
    let name = info
        .get("name")
        .and_then(|v| v.str())
        .ok_or_else(|| "no name".to_owned())?;
    let piece_length = info
        .get("piece length")
        .and_then(|v| v.int())
        .ok_or_else(|| "no piece length".to_owned())? as u64;
    // Single file: length directly; multi-file: files list with path joins.
    let mut files = Vec::new();
    if let Some(length) = info.get("length").and_then(|v| v.int()) {
        files.push(TorrentFile {
            path: name.clone(),
            length: length as u64,
        });
    } else if let Some(file_list) = info.get("files").and_then(|v| v.list()) {
        for entry in file_list {
            let Some(d) = entry.dict() else { continue };
            let Some(length) = d.get("length").and_then(|v| v.int()) else {
                continue;
            };
            let mut segs = Vec::new();
            if let Some(path_segs) = d.get("path").and_then(|v| v.list()) {
                for seg in path_segs {
                    if let Some(s) = seg.str() {
                        segs.push(s);
                    }
                }
            }
            let path = if segs.is_empty() {
                name.clone()
            } else {
                segs.join("/")
            };
            files.push(TorrentFile {
                path,
                length: length as u64,
            });
        }
    } else {
        return Err("no length or files in info".into());
    }
    let total: u64 = files.iter().map(|f| f.length).sum();
    let piece_count = if piece_length == 0 {
        0
    } else {
        total.div_ceil(piece_length)
    };
    Ok(TorrentInfo {
        name,
        files,
        total,
        piece_length,
        piece_count,
    })
}

/// A Magnet link's display data (header + params, no peers yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MagnetInfo {
    pub info_hash: String,
    pub name: Option<String>,
    pub trackers: Vec<String>,
}

/// Parse `magnet:?xt=urn:btih:<infohash>&dn=<name>&tr=<tracker>…`.
pub fn parse_magnet(raw: &str) -> Result<MagnetInfo, String> {
    let raw = raw.trim();
    if !raw.to_ascii_lowercase().starts_with("magnet:") {
        return Err("not a magnet link".into());
    }
    let query = raw.split('?').nth(1).unwrap_or("");
    let mut info_hash = None;
    let mut name = None;
    let mut trackers = Vec::new();
    for pair in query.split('&') {
        let Some((k, v)) = pair.split_once('=') else {
            continue;
        };
        match k {
            "xt" if v.to_ascii_lowercase().starts_with("urn:btih:") => {
                info_hash = Some(v[9..].to_owned());
            }
            "dn" => name = Some(v.to_owned()),
            "tr" => trackers.push(v.to_owned()),
            _ => {}
        }
    }
    let info_hash = info_hash.ok_or_else(|| "magnet without btih hash".to_owned())?;
    if info_hash.len() != 40 && info_hash.len() != 32 {
        // 40 hex chars (SHA-1), or 32 base32 chars.
        return Err("magnet hash is neither 40 hex nor 32 base32".into());
    }
    Ok(MagnetInfo {
        info_hash,
        name,
        trackers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bencode_reference_vector() {
        let input = b"d3:cow3:moo4:spam4:eggse";
        let (value, _) = decode(input, 0).unwrap();
        let dict = value.dict().unwrap();
        assert_eq!(dict.get("cow").unwrap().str().unwrap(), "moo");
        assert_eq!(dict.get("spam").unwrap().str().unwrap(), "eggs");
    }

    #[test]
    fn torrent_single_file() {
        // d4:infod6:lengthi5e4:name8:file.txt12:piece lengthi16384eee
        let data = b"d4:infod6:lengthi5e4:name8:file.txt12:piece lengthi16384eee";
        let info = parse_torrent(data).unwrap();
        assert_eq!(info.name, "file.txt");
        assert_eq!(info.total, 5);
        assert_eq!(info.piece_count, 1);
    }

    #[test]
    fn magnet_parses() {
        let m =
            parse_magnet("magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567&dn=Hello")
                .unwrap();
        assert_eq!(m.info_hash.len(), 40);
        assert_eq!(m.name.as_deref(), Some("Hello"));
    }

    #[test]
    fn magnet_needs_hash() {
        assert!(parse_magnet("magnet:?dn=x").is_err());
    }
}
