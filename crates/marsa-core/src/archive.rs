//! Smart Archive Manager (v1): what an archive holds, and extracting it.
//!
//! Formats: ZIP (stored or deflated), TAR (USTAR, with GNU's long-name
//! entry) and `.tar.gz`. TAR is read here from the published header
//! (POSIX 1003.1-1988 §10.1, and GNU tar's `L` entry) with no new
//! dependency. RAR is **not** read: the unRAR licence forbids reverse
//! engineering and works derived from it, so no part of it is used here.
//! 7z is not read yet either.
//!
//! Names come from the archive and are therefore untrusted. Every path
//! part goes through [`sanitize_file_name`], `..` is refused, links in the
//! archive are never made, and a file is written only through
//! [`marsa_platform::fs::open_regular_nofollow`] or a `create_new`: a link
//! planted in the destination folder cannot send a byte outside it.

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::sanitize::{FALLBACK_NAME, MAX_NAME_BYTES, sanitize_file_name};

/// One block of a TAR: headers and their padding are counted in these.
const BLOCK: usize = 512;

/// How many entries an archive may hold before it is called damaged. An
/// index is kept in memory, so this bounds it.
const MAX_ENTRIES: usize = 100_000;

/// The suffix of the folder an archive is extracted into.
const EXTRACTED: &str = "_extracted";

/// One entry of an archive, as the pages show it: the archive's own name,
/// its size, and whether it is a folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
}

/// The archive formats Marsa reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Zip,
    Tar,
    TarGz,
}

/// Errors from [`list`] and [`extract`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("not an archive Marsa reads (zip, tar, tar.gz)")]
    UnknownFormat,
    #[error("{name:?}: a path that would leave the folder")]
    UnsafePath { name: String },
    #[error("{}: not a folder Marsa writes in", path.display())]
    NotAFolder { path: PathBuf },
    #[error("damaged archive: {0}")]
    Damaged(String),
}

/// The format of `path`, read from its first bytes rather than its name:
/// a `.zip` that is really a TAR is read as a TAR.
pub fn format_of(path: &Path) -> io::Result<Option<Format>> {
    let mut head = [0u8; BLOCK];
    let mut file = File::open(path)?;
    let mut filled = 0;
    while filled < head.len() {
        let read = file.read(&mut head[filled..])?;
        if read == 0 {
            break;
        }
        filled += read;
    }
    Ok(format_of_head(&head[..filled]))
}

fn format_of_head(head: &[u8]) -> Option<Format> {
    if head.starts_with(b"PK\x03\x04") || head.starts_with(b"PK\x05\x06") {
        return Some(Format::Zip);
    }
    if head.starts_with(&[0x1f, 0x8b]) {
        return Some(Format::TarGz);
    }
    if head.len() >= 262 && &head[257..262] == b"ustar" {
        return Some(Format::Tar);
    }
    // An empty TAR: nothing but the two end blocks.
    if head.len() >= 2 * BLOCK && head.iter().all(|byte| *byte == 0) {
        return Some(Format::Tar);
    }
    None
}

/// The folder `archive` is extracted into, next to it: `<name>_extracted`,
/// with `.tar.gz` taken as one extension. Like every name Marsa makes, the
/// result stays within [`MAX_NAME_BYTES`], shortened on a character
/// boundary.
pub fn folder_for(archive: &Path) -> PathBuf {
    let parent = archive.parent().unwrap_or(Path::new("."));
    let name = archive
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let lower = name.to_lowercase();
    let stem = if lower.ends_with(".tar.gz") {
        &name[..name.len() - ".tar.gz".len()]
    } else {
        match name.rfind('.') {
            Some(at) if at > 0 => &name[..at],
            _ => name.as_str(),
        }
    };
    parent.join(with_suffix(stem, EXTRACTED))
}

/// `stem` followed by `suffix`, shortened so the whole name fits
/// [`MAX_NAME_BYTES`]. A stem that is left empty (a name that is only an
/// extension) becomes [`FALLBACK_NAME`].
fn with_suffix(stem: &str, suffix: &str) -> String {
    let room = MAX_NAME_BYTES.saturating_sub(suffix.len());
    let mut end = stem.len().min(room);
    while end > 0 && !stem.is_char_boundary(end) {
        end -= 1;
    }
    let stem = if end == 0 {
        FALLBACK_NAME
    } else {
        &stem[..end]
    };
    format!("{stem}{suffix}")
}

/// Lists what `path` holds, without writing anything.
pub fn list(path: &Path) -> Result<Vec<Entry>, Error> {
    let format = format_of(path)?.ok_or(Error::UnknownFormat)?;
    let mut entries = Vec::new();
    match format {
        Format::Zip => {
            let mut zip = open_zip(path)?;
            if zip.len() > MAX_ENTRIES {
                return Err(Error::Damaged(format!("more than {MAX_ENTRIES} entries")));
            }
            for index in 0..zip.len() {
                let entry = zip
                    .by_index(index)
                    .map_err(|e| Error::Damaged(format!("entry {index}: {e}")))?;
                entries.push(Entry {
                    name: entry
                        .name()
                        .map(|name| name.into_owned())
                        .unwrap_or_else(|_| format!("entry {index}")),
                    size: entry.size(),
                    is_dir: entry.is_dir(),
                });
            }
        }
        Format::Tar => tar_list(File::open(path)?, &mut entries)?,
        Format::TarGz => tar_list(gzip(path)?, &mut entries)?,
    }
    Ok(entries)
}

/// Extracts `path` into `dest`, making it if it is not there. A name that
/// would leave `dest` stops the extraction with [`Error::UnsafePath`].
pub fn extract(path: &Path, dest: &Path) -> Result<(), Error> {
    let format = format_of(path)?.ok_or(Error::UnknownFormat)?;
    ensure_dest(dest)?;
    match format {
        Format::Zip => {
            let mut zip = open_zip(path)?;
            if zip.len() > MAX_ENTRIES {
                return Err(Error::Damaged(format!("more than {MAX_ENTRIES} entries")));
            }
            for index in 0..zip.len() {
                let mut entry = zip
                    .by_index(index)
                    .map_err(|e| Error::Damaged(format!("entry {index}: {e}")))?;
                let name = entry
                    .name()
                    .map(|name| name.into_owned())
                    .unwrap_or_default();
                // A ZIP entry that is a link is written as a file holding
                // the text of its target, never made a link.
                write_entry(dest, &name, entry.is_dir(), &mut entry)?;
            }
            Ok(())
        }
        Format::Tar => tar_extract(File::open(path)?, dest),
        Format::TarGz => tar_extract(gzip(path)?, dest),
    }
}

fn open_zip(path: &Path) -> Result<zip::ZipArchive<File>, Error> {
    zip::ZipArchive::new(File::open(path)?).map_err(|e| Error::Damaged(format!("not a zip: {e}")))
}

fn gzip(path: &Path) -> Result<flate2::read::GzDecoder<File>, Error> {
    Ok(flate2::read::GzDecoder::new(File::open(path)?))
}

fn tar_list<R: Read>(reader: R, entries: &mut Vec<Entry>) -> Result<(), Error> {
    walk(reader, |name, size, is_dir, _body| {
        entries.push(Entry {
            name: name.to_owned(),
            size,
            is_dir,
        });
        Ok(())
    })
}

fn tar_extract<R: Read>(reader: R, dest: &Path) -> Result<(), Error> {
    walk(reader, |name, _size, is_dir, body| {
        write_entry(dest, name, is_dir, body)
    })
}

/// Walks a TAR, calling `each` for every file and folder with a reader of
/// its body. Links, devices and pipes are skipped: they are never made.
fn walk<R: Read, F>(mut reader: R, mut each: F) -> Result<(), Error>
where
    F: FnMut(&str, u64, bool, &mut dyn Read) -> Result<(), Error>,
{
    let mut long_name: Option<String> = None;
    let mut index = 0usize;
    loop {
        let mut header = [0u8; BLOCK];
        if !read_block(&mut reader, &mut header)? {
            // A file that ends without the two end blocks: what it had is
            // still readable, and the checksum of every header passed.
            return Ok(());
        }
        if header.iter().all(|byte| *byte == 0) {
            return Ok(());
        }
        index += 1;
        if index > MAX_ENTRIES {
            return Err(Error::Damaged(format!("more than {MAX_ENTRIES} entries")));
        }
        let stored = number(&header[148..156])
            .ok_or_else(|| Error::Damaged(format!("entry {index}: unreadable checksum")))?;
        let sum: u64 = header
            .iter()
            .enumerate()
            .map(|(at, byte)| {
                if (148..156).contains(&at) {
                    u64::from(b' ')
                } else {
                    u64::from(*byte)
                }
            })
            .sum();
        if stored != sum {
            return Err(Error::Damaged(format!("entry {index}: checksum mismatch")));
        }
        let size = number(&header[124..136])
            .ok_or_else(|| Error::Damaged(format!("entry {index}: unreadable size")))?;
        let name = long_name.take().unwrap_or_else(|| name_of(&header));
        match header[156] {
            // GNU: the body of this entry is the name of the next one.
            b'L' => long_name = Some(read_name(&mut reader, size, index)?),
            // A long link name, and the pax headers: nothing to write. Their
            // body is padded like any other, or the next header is read from
            // the middle of that padding.
            b'K' | b'x' | b'g' => skip(&mut reader, size.saturating_add(pad(size)), index)?,
            // A file.
            b'0' | 0 => {
                let mut body = (&mut reader).take(size);
                each(&name, size, false, &mut body)?;
                io::copy(&mut body, &mut io::sink())
                    .map_err(|e| Error::Damaged(format!("entry {index}: {e}")))?;
                skip(&mut reader, pad(size), index)?;
            }
            // A folder.
            b'5' => {
                each(&name, 0, true, &mut io::empty())?;
                skip(&mut reader, size.saturating_add(pad(size)), index)?;
            }
            // Devices, pipes and links: skipped whole, padding included.
            _ => skip(&mut reader, size.saturating_add(pad(size)), index)?,
        }
    }
}

/// Reads a whole header block. `Ok(false)` at a clean end of file.
fn read_block<R: Read>(reader: &mut R, block: &mut [u8; BLOCK]) -> Result<bool, Error> {
    let mut filled = 0;
    while filled < BLOCK {
        let read = reader.read(&mut block[filled..])?;
        if read == 0 {
            return if filled == 0 {
                Ok(false)
            } else {
                Err(Error::Damaged("cut in the middle of a header".into()))
            };
        }
        filled += read;
    }
    Ok(true)
}

/// Reads the body of a GNU long-name entry as a name, and its padding.
fn read_name<R: Read>(reader: &mut R, size: u64, index: usize) -> Result<String, Error> {
    if size > 1 << 20 {
        return Err(Error::Damaged(format!(
            "entry {index}: a name of {size} bytes"
        )));
    }
    let mut bytes = vec![0u8; size as usize];
    reader
        .read_exact(&mut bytes)
        .map_err(|e| Error::Damaged(format!("entry {index}: {e}")))?;
    skip(reader, pad(size), index)?;
    let text = String::from_utf8_lossy(&bytes);
    Ok(text.trim_end_matches('\0').to_owned())
}

/// Reads and drops `count` bytes, refusing a file that stops early.
fn skip<R: Read>(reader: &mut R, count: u64, index: usize) -> Result<(), Error> {
    let copied = io::copy(&mut reader.take(count), &mut io::sink())
        .map_err(|e| Error::Damaged(format!("entry {index}: {e}")))?;
    if copied != count {
        return Err(Error::Damaged(format!("entry {index}: cut short")));
    }
    Ok(())
}

/// The bytes that pad a body of `size` bytes up to a whole block.
fn pad(size: u64) -> u64 {
    (BLOCK as u64 - size % BLOCK as u64) % BLOCK as u64
}

/// A number from a header field: octal as POSIX writes it, or GNU's
/// base-256 when the value does not fit the octal field.
fn number(field: &[u8]) -> Option<u64> {
    if field.first().is_some_and(|byte| byte & 0x80 != 0) {
        let mut value = u64::from(field[0] & 0x7f);
        for byte in &field[1..] {
            value = value.checked_mul(256)?.checked_add(u64::from(*byte))?;
        }
        return Some(value);
    }
    let mut value = 0u64;
    for byte in field {
        match byte {
            b'0'..=b'7' => {
                value = value.checked_mul(8)?.checked_add(u64::from(byte - b'0'))?;
            }
            0 | b' ' => {}
            _ => return None,
        }
    }
    Some(value)
}

/// An entry's name from its header: POSIX ustar splits a long path into a
/// prefix and a name.
fn name_of(header: &[u8; BLOCK]) -> String {
    let name = text(&header[0..100]);
    if &header[257..263] == b"ustar\0" {
        let prefix = text(&header[345..500]);
        if !prefix.is_empty() {
            return format!("{prefix}/{name}");
        }
    }
    name
}

fn text(field: &[u8]) -> String {
    let end = field
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(field.len());
    String::from_utf8_lossy(&field[..end]).into_owned()
}

/// Where an entry goes, below `dest`: `Ok(None)` for an entry that names
/// nothing (`.`, `./`), and [`Error::UnsafePath`] for one that would leave.
fn relative_path(name: &str) -> Result<Option<PathBuf>, Error> {
    let mut relative = PathBuf::new();
    for part in name.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => {
                return Err(Error::UnsafePath {
                    name: name.to_owned(),
                });
            }
            part => relative.push(sanitize_file_name(part)),
        }
    }
    Ok((!relative.as_os_str().is_empty()).then_some(relative))
}

fn write_entry(dest: &Path, name: &str, is_dir: bool, body: &mut dyn Read) -> Result<(), Error> {
    let Some(relative) = relative_path(name)? else {
        return Ok(());
    };
    if is_dir {
        return ensure_below(dest, &relative);
    }
    if let Some(parent) = relative.parent() {
        ensure_below(dest, parent)?;
    }
    let mut file = create_target(&dest.join(&relative))?;
    io::copy(body, &mut file)?;
    Ok(())
}

/// The folder the archive is extracted into: made if it is not there, and
/// refused if something else (a link, a file) is in its place.
fn ensure_dest(dest: &Path) -> Result<(), Error> {
    match std::fs::symlink_metadata(dest) {
        Ok(meta) if meta.is_dir() => Ok(()),
        Ok(_) => Err(Error::NotAFolder {
            path: dest.to_path_buf(),
        }),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            std::fs::create_dir_all(dest).map_err(Error::Io)
        }
        Err(err) => Err(Error::Io(err)),
    }
}

/// Makes the folders of `relative` below `dest`, one at a time, so a link
/// in the way is seen rather than followed.
fn ensure_below(dest: &Path, relative: &Path) -> Result<(), Error> {
    let mut current = dest.to_path_buf();
    for part in relative.components() {
        current.push(part.as_os_str());
        match std::fs::symlink_metadata(&current) {
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => {
                return Err(Error::NotAFolder {
                    path: current.clone(),
                });
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                std::fs::create_dir(&current).map_err(Error::Io)?;
            }
            Err(err) => return Err(Error::Io(err)),
        }
    }
    Ok(())
}

/// A file to write an entry's bytes into: made if it is not there, and an
/// existing one opened only if it is a regular file with a single name, then
/// emptied — so extracting again replaces a file instead of appending to it,
/// and never writes through a link.
fn create_target(path: &Path) -> Result<File, Error> {
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(file) => Ok(file),
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            let file = marsa_platform::fs::open_regular_nofollow(path, true)?;
            file.set_len(0)?;
            Ok(file)
        }
        Err(err) => Err(Error::Io(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    /// A USTAR header with its checksum filled in, as POSIX 1003.1-1988
    /// writes it.
    fn header(name: &str, size: u64, kind: u8) -> [u8; BLOCK] {
        let mut block = [0u8; BLOCK];
        block[..name.len()].copy_from_slice(name.as_bytes());
        block[100..108].copy_from_slice(b"0000644\0");
        block[108..116].copy_from_slice(b"0000000\0");
        block[116..124].copy_from_slice(b"0000000\0");
        block[124..136].copy_from_slice(format!("{size:011o}\0").as_bytes());
        block[136..148].copy_from_slice(b"00000000000\0");
        block[148..156].copy_from_slice(b"        ");
        block[156] = kind;
        block[257..263].copy_from_slice(b"ustar\0");
        block[263..265].copy_from_slice(b"00");
        let sum: u64 = block.iter().map(|byte| u64::from(*byte)).sum();
        block[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
        block
    }

    fn file_entry(name: &str, body: &[u8]) -> Vec<u8> {
        let mut out = header(name, body.len() as u64, b'0').to_vec();
        out.extend_from_slice(body);
        out.resize(out.len() + pad(body.len() as u64) as usize, 0);
        out
    }

    fn dir_entry(name: &str) -> Vec<u8> {
        header(name, 0, b'5').to_vec()
    }

    /// Fills in the checksum of a header that a test changed by hand, as a
    /// real writer would.
    fn reseal(block: &mut [u8]) {
        block[148..156].copy_from_slice(b"        ");
        let sum: u64 = block.iter().map(|byte| u64::from(*byte)).sum();
        block[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
    }

    fn archive(entries: Vec<Vec<u8>>) -> Vec<u8> {
        let mut out: Vec<u8> = entries.concat();
        out.extend_from_slice(&[0u8; 2 * BLOCK]);
        out
    }

    #[test]
    fn the_extract_folder_is_named_after_the_archive() {
        let dir = Path::new("C:/downloads");
        assert_eq!(
            folder_for(&dir.join("file.zip")),
            dir.join("file_extracted")
        );
        assert_eq!(
            folder_for(&dir.join("backup.tar.gz")),
            dir.join("backup_extracted")
        );
        assert_eq!(
            folder_for(&dir.join("BACKUP.TAR.GZ")),
            dir.join("BACKUP_extracted")
        );
        assert_eq!(folder_for(&dir.join("notes")), dir.join("notes_extracted"));
        assert_eq!(
            folder_for(&dir.join(".gitignore")),
            dir.join(".gitignore_extracted")
        );
    }

    /// A folder name Marsa makes stays within the same limit as every other
    /// name it makes, with a name at the limit and Arabic letters (more
    /// than one byte each) cut on a character boundary.
    #[test]
    fn the_extract_folder_stays_within_the_name_limit() {
        for extra in 0..4 {
            let stem = format!("{}{}", "م".repeat(98), "a".repeat(extra));
            for extension in [".zip", ".tar", ".tar.gz"] {
                let name = format!("{stem}{extension}");
                let folder = folder_for(Path::new("C:/downloads").join(&name).as_path());
                let made = folder.file_name().unwrap().to_string_lossy().into_owned();
                assert!(made.len() <= MAX_NAME_BYTES, "{name} -> {made}");
                assert!(made.ends_with("_extracted"), "{name} -> {made}");
                assert!(made.starts_with('م'), "{name} -> {made}");
            }
        }
    }

    #[test]
    fn a_tar_lists_and_extracts_its_folders_and_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.tar");
        std::fs::write(
            &path,
            archive(vec![
                dir_entry("docs/"),
                file_entry("docs/readme.txt", b"hello"),
                file_entry("notes.txt", b"marsa"),
            ]),
        )
        .unwrap();

        assert_eq!(format_of(&path).unwrap(), Some(Format::Tar));
        assert_eq!(
            list(&path).unwrap(),
            vec![
                Entry {
                    name: "docs/".into(),
                    size: 0,
                    is_dir: true,
                },
                Entry {
                    name: "docs/readme.txt".into(),
                    size: 5,
                    is_dir: false,
                },
                Entry {
                    name: "notes.txt".into(),
                    size: 5,
                    is_dir: false,
                },
            ]
        );

        let dest = dir.path().join("out");
        extract(&path, &dest).unwrap();
        assert_eq!(
            std::fs::read(dest.join("docs/readme.txt")).unwrap(),
            b"hello"
        );
        assert_eq!(std::fs::read(dest.join("notes.txt")).unwrap(), b"marsa");
    }

    #[test]
    fn a_tar_gz_is_read_through_its_gzip_layer() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.tar.gz");
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder
            .write_all(&archive(vec![file_entry("inner.txt", b"gz")]))
            .unwrap();
        std::fs::write(&path, encoder.finish().unwrap()).unwrap();

        assert_eq!(format_of(&path).unwrap(), Some(Format::TarGz));
        assert_eq!(list(&path).unwrap()[0].name, "inner.txt");

        let dest = dir.path().join("out");
        extract(&path, &dest).unwrap();
        assert_eq!(std::fs::read(dest.join("inner.txt")).unwrap(), b"gz");
    }

    #[test]
    fn a_zip_lists_and_extracts_through_the_same_door() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.zip");
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        writer
            .start_file("z/inner.txt", zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"zip").unwrap();
        std::fs::write(&path, writer.finish().unwrap().into_inner()).unwrap();

        assert_eq!(format_of(&path).unwrap(), Some(Format::Zip));
        assert_eq!(
            list(&path).unwrap(),
            vec![Entry {
                name: "z/inner.txt".into(),
                size: 3,
                is_dir: false,
            }]
        );

        let dest = dir.path().join("out");
        extract(&path, &dest).unwrap();
        assert_eq!(std::fs::read(dest.join("z/inner.txt")).unwrap(), b"zip");
    }

    /// A name that walks out of the folder stops the extraction, and
    /// nothing outside is touched; an absolute name lands inside instead.
    #[test]
    fn a_name_that_would_leave_the_folder_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let outside = dir.path().join("outside.txt");
        let dest = dir.path().join("out");

        for name in ["../outside.txt", "..\\outside.txt", "a/../../outside.txt"] {
            let path = dir.path().join("evil.tar");
            std::fs::write(&path, archive(vec![file_entry(name, b"no")])).unwrap();
            let error = extract(&path, &dest).unwrap_err();
            assert!(matches!(error, Error::UnsafePath { .. }), "{name}: {error}");
            assert!(!outside.exists(), "{name}");
        }

        let path = dir.path().join("absolute.tar");
        std::fs::write(&path, archive(vec![file_entry("/etc/passwd", b"no")])).unwrap();
        extract(&path, &dest).unwrap();
        assert_eq!(std::fs::read(dest.join("etc/passwd")).unwrap(), b"no");
    }

    /// A pax header (bsdtar writes one for long names and the like) is
    /// skipped whole, padding included, so the entry after it is read at the
    /// right place rather than in the middle of the padding.
    #[test]
    fn a_pax_header_is_skipped_without_losing_the_next_entry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pax.tar");
        let record = b"path=notes.txt\n";
        let mut pax = header("PaxHeaders/notes.txt", record.len() as u64, b'x').to_vec();
        pax.extend_from_slice(record);
        pax.resize(pax.len() + pad(record.len() as u64) as usize, 0);
        std::fs::write(&path, archive(vec![pax, file_entry("notes.txt", b"after")])).unwrap();

        assert_eq!(list(&path).unwrap().len(), 1);
        let dest = dir.path().join("out");
        extract(&path, &dest).unwrap();
        assert_eq!(std::fs::read(dest.join("notes.txt")).unwrap(), b"after");
    }

    /// A link in the archive is never made a link: nothing is created.
    #[test]
    fn a_link_in_the_archive_is_not_made() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("links.tar");
        let mut link = header("passwd", 0, b'2').to_vec();
        link[157..168].copy_from_slice(b"/etc/passwd");
        reseal(&mut link);
        let mut hard = header("hard", 0, b'1').to_vec();
        hard[157..165].copy_from_slice(b"passwd\0\0");
        reseal(&mut hard);
        std::fs::write(&path, archive(vec![link, hard])).unwrap();

        let dest = dir.path().join("out");
        extract(&path, &dest).unwrap();
        assert!(!dest.join("passwd").exists());
        assert!(!dest.join("hard").exists());
    }

    #[test]
    fn a_header_with_a_broken_checksum_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("broken.tar");
        let mut bytes = file_entry("a.txt", b"x");
        bytes[0] = b'b';
        std::fs::write(&path, archive(vec![bytes])).unwrap();

        let error = extract(&path, &dir.path().join("out")).unwrap_err();
        assert!(
            matches!(&error, Error::Damaged(reason) if reason.contains("checksum")),
            "{error}"
        );
    }

    /// A symbolic link planted in the destination folder is not written
    /// through: the file it points at keeps its bytes. Fails if the link
    /// cannot be made at all (no silent skip).
    #[test]
    fn a_symbolic_link_in_the_folder_is_not_written_through() {
        let dir = tempfile::tempdir().unwrap();
        let outside = dir.path().join("outside.txt");
        std::fs::write(&outside, b"untouched").unwrap();
        let dest = dir.path().join("out");
        std::fs::create_dir(&dest).unwrap();
        marsa_platform::fs::symlink_file(&outside, &dest.join("a.txt"))
            .expect("a symbolic link in the test's folder");

        let path = dir.path().join("a.tar");
        std::fs::write(&path, archive(vec![file_entry("a.txt", b"mine")])).unwrap();
        let error = extract(&path, &dest).unwrap_err();
        assert!(matches!(error, Error::Io(_)), "{error}");
        assert_eq!(std::fs::read(&outside).unwrap(), b"untouched");
    }

    /// The same for a hard link: refused, and the file outside keeps its
    /// bytes.
    #[test]
    fn a_hard_link_in_the_folder_is_not_written_through() {
        let dir = tempfile::tempdir().unwrap();
        let outside = dir.path().join("outside.txt");
        std::fs::write(&outside, b"untouched").unwrap();
        let dest = dir.path().join("out");
        std::fs::create_dir(&dest).unwrap();
        std::fs::hard_link(&outside, dest.join("a.txt")).unwrap();

        let path = dir.path().join("a.tar");
        std::fs::write(&path, archive(vec![file_entry("a.txt", b"mine")])).unwrap();
        let error = extract(&path, &dest).unwrap_err();
        assert!(matches!(error, Error::Io(_)), "{error}");
        assert_eq!(std::fs::read(&outside).unwrap(), b"untouched");
    }

    /// Extracting again replaces a file instead of appending to it.
    #[test]
    fn extracting_again_replaces_a_file_rather_than_appending_to_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.tar");
        let dest = dir.path().join("out");

        std::fs::write(&path, archive(vec![file_entry("a.txt", b"longer")])).unwrap();
        extract(&path, &dest).unwrap();
        std::fs::write(&path, archive(vec![file_entry("a.txt", b"no")])).unwrap();
        extract(&path, &dest).unwrap();

        assert_eq!(std::fs::read(dest.join("a.txt")).unwrap(), b"no");
    }
}
