//! Bound the ZIP index before the ZIP library allocates it. During indexing the
//! reader exposes only validated directory/footer bytes and fixed local headers;
//! malformed directories cannot trigger fallback scans into arbitrary payloads.
use std::cell::Cell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::rc::Rc;

use anyhow::{ensure, Context, Result};
use zip::ZipArchive;

use super::inspect_limits::{check, InspectLimits};

const END: &[u8] = b"PK\x05\x06";
const END64: &[u8] = b"PK\x06\x06";
const LOCATOR: &[u8] = b"PK\x06\x07";

pub(crate) fn open(path: &Path, limits: &InspectLimits) -> Result<ZipArchive<IndexedFile>> {
    let mut file = File::open(path).with_context(|| format!("open APKG {}", path.display()))?;
    let length = file.metadata()?.len();
    check("archive_bytes", None, limits.max_archive_bytes, length)?;
    ensure!(length >= 22, "truncated ZIP footer");
    let tail_len = length.min(65_535 + 22) as usize;
    let tail_start = length - tail_len as u64;
    let mut tail = vec![0; tail_len];
    file.seek(SeekFrom::Start(tail_start))?;
    file.read_exact(&mut tail)?;
    let end = (0..=tail_len - 22)
        .rev()
        .find(|&i| &tail[i..i + 4] == END && i + 22 + u16_at(&tail, i + 20) as usize == tail_len)
        .context("ZIP footer must terminate at end of file")?;
    let end_pos = tail_start + end as u64;
    let footer = &tail[end..];
    ensure!(
        u16_at(footer, 4) == 0 && u16_at(footer, 6) == 0,
        "multi-disk ZIP is unsupported"
    );
    let mut count = u16_at(footer, 10) as u64;
    ensure!(
        u16_at(footer, 8) as u64 == count,
        "inconsistent ZIP entry counts"
    );
    let mut directory_size = u32_at(footer, 12) as u64;
    let mut directory_start = u32_at(footer, 16) as u64;
    let mut directory_end = end_pos;
    let mut footer_positions = vec![end_pos];
    // ZIP64 is allowed even when legacy fields are not saturated.
    if end_pos >= 20 {
        let mut locator = [0; 20];
        file.seek(SeekFrom::Start(end_pos - 20))?;
        file.read_exact(&mut locator)?;
        if &locator[..4] == LOCATOR {
            ensure!(
                u32_at(&locator, 4) == 0 && u32_at(&locator, 16) == 1,
                "multi-disk ZIP64 is unsupported"
            );
            let pos = u64_at(&locator, 8);
            ensure!(
                pos <= end_pos - 20 && end_pos - 20 - pos >= 56,
                "invalid ZIP64 footer offset"
            );
            let mut record = [0; 56];
            file.seek(SeekFrom::Start(pos))?;
            file.read_exact(&mut record)?;
            ensure!(&record[..4] == END64, "invalid ZIP64 footer");
            let size = u64_at(&record, 4);
            check(
                "central_directory_bytes",
                None,
                limits.max_central_directory_bytes,
                size,
            )?;
            ensure!(
                size >= 44 && size.checked_add(12) == Some(end_pos - 20 - pos),
                "invalid ZIP64 footer size"
            );
            ensure!(
                u32_at(&record, 16) == 0 && u32_at(&record, 20) == 0,
                "multi-disk ZIP64 is unsupported"
            );
            count = u64_at(&record, 32);
            check("entries", None, limits.max_entries, count)?;
            ensure!(
                u64_at(&record, 24) == count,
                "inconsistent ZIP64 entry counts"
            );
            directory_size = u64_at(&record, 40);
            directory_start = u64_at(&record, 48);
            directory_end = pos;
            footer_positions.extend([pos, end_pos - 20]);
        }
    }
    check("entries", None, limits.max_entries, count)?;
    check(
        "central_directory_bytes",
        None,
        limits.max_central_directory_bytes,
        directory_size,
    )?;
    ensure!(
        directory_start.checked_add(directory_size) == Some(directory_end),
        "invalid ZIP directory extent (prefixed/ambiguous archives are unsupported)"
    );
    let snapshot_size = length - directory_start;
    check(
        "central_directory_bytes",
        None,
        limits
            .max_central_directory_bytes
            .saturating_mul(2)
            .saturating_add(65_633),
        snapshot_size,
    )?;
    let mut snapshot = vec![0; usize::try_from(snapshot_size)?];
    file.seek(SeekFrom::Start(directory_start))?;
    file.read_exact(&mut snapshot)?;
    // Never allow the library's permissive footer fallback to pick a second,
    // unvalidated count from a filename, comment, extension or payload.
    for (i, magic) in snapshot.windows(4).enumerate() {
        if matches!(magic, END | END64 | LOCATOR) {
            ensure!(
                footer_positions.contains(&(directory_start + i as u64)),
                "ambiguous ZIP footer signature"
            );
        }
    }
    let mut local_headers = BTreeMap::new();
    let mut offset = 0usize;
    for _ in 0..count {
        let fixed = snapshot
            .get(offset..offset + 46)
            .context("truncated ZIP central header")?;
        ensure!(&fixed[..4] == b"PK\x01\x02", "invalid ZIP central header");
        let name_len = u16_at(fixed, 28) as usize;
        let extra_len = u16_at(fixed, 30) as usize;
        let comment_len = u16_at(fixed, 32) as usize;
        let next = offset
            .checked_add(46 + name_len + extra_len + comment_len)
            .context("ZIP index overflow")?;
        ensure!(
            next as u64 <= directory_size,
            "ZIP central header exceeds directory"
        );
        let mut local_offset = u32_at(fixed, 42) as u64;
        let extra = &snapshot[offset + 46 + name_len..offset + 46 + name_len + extra_len];
        let mut extra_offset = 0;
        while extra_offset < extra.len() {
            ensure!(extra.len() - extra_offset >= 4, "truncated ZIP extra field");
            let id = u16_at(extra, extra_offset);
            let size = u16_at(extra, extra_offset + 2) as usize;
            let data = extra
                .get(extra_offset + 4..extra_offset + 4 + size)
                .context("invalid ZIP extra field size")?;
            if id == 1 {
                // Match zip's ZIP64 extended-information interpretation.
                let skip = 8
                    * (usize::from(size >= 24 || u32_at(fixed, 24) == u32::MAX)
                        + usize::from(size >= 24 || u32_at(fixed, 20) == u32::MAX));
                if size >= 24 || local_offset == u32::MAX as u64 {
                    local_offset = u64::from_le_bytes(
                        data.get(skip..skip + 8)
                            .context("truncated ZIP64 offset")?
                            .try_into()
                            .unwrap(),
                    );
                }
            }
            extra_offset += 4 + size;
        }
        ensure!(
            local_offset
                .checked_add(30)
                .is_some_and(|end| end <= directory_start),
            "invalid local ZIP header offset"
        );
        let mut header = [0; 30];
        file.seek(SeekFrom::Start(local_offset))?;
        file.read_exact(&mut header)?;
        ensure!(&header[..4] == b"PK\x03\x04", "invalid local ZIP header");
        ensure!(
            !header
                .windows(4)
                .any(|m| matches!(m, END | END64 | LOCATOR)),
            "ambiguous local ZIP header"
        );
        local_headers.insert(local_offset, header);
        offset = next;
    }
    ensure!(
        offset as u64 == directory_size,
        "ZIP entry count does not match directory"
    );
    let indexing = Rc::new(Cell::new(true));
    let reader = IndexedFile {
        file,
        length,
        position: 0,
        snapshot,
        directory_start,
        local_headers,
        indexing: indexing.clone(),
        index_read_remaining: snapshot_size
            .saturating_mul(4)
            .saturating_add(count.saturating_mul(64))
            .saturating_add(65_536),
    };
    let archive = ZipArchive::with_config(
        zip::read::Config {
            archive_offset: zip::read::ArchiveOffset::Known(0),
        },
        reader,
    )?;
    indexing.set(false);
    check("entries", None, limits.max_entries, archive.len() as u64)?;
    Ok(archive)
}

pub(crate) struct IndexedFile {
    file: File,
    length: u64,
    position: u64,
    snapshot: Vec<u8>,
    directory_start: u64,
    local_headers: BTreeMap<u64, [u8; 30]>,
    indexing: Rc<Cell<bool>>,
    index_read_remaining: u64,
}

impl Read for IndexedFile {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() || self.position >= self.length {
            return Ok(0);
        }
        let mut size = buffer
            .len()
            .min((self.length - self.position).min(usize::MAX as u64) as usize);
        if self.indexing.get() {
            if size as u64 > self.index_read_remaining {
                return Err(io::Error::other("ZIP index fallback read budget exhausted"));
            }
            self.index_read_remaining -= size as u64;
        }
        if self.position >= self.directory_start {
            let start = (self.position - self.directory_start) as usize;
            buffer[..size].copy_from_slice(&self.snapshot[start..start + size]);
        } else if self.indexing.get() {
            size = size.min((self.directory_start - self.position).min(usize::MAX as u64) as usize);
            if let Some((&start, header)) = self
                .local_headers
                .range(..=self.position)
                .next_back()
                .filter(|(start, _)| self.position < **start + 30)
            {
                let offset = (self.position - start) as usize;
                size = size.min(30 - offset);
                buffer[..size].copy_from_slice(&header[offset..offset + size]);
            } else {
                // Footer searches see no signatures in unvalidated payloads.
                buffer[..size].fill(0);
            }
        } else {
            size = size.min((self.directory_start - self.position).min(usize::MAX as u64) as usize);
            self.file.seek(SeekFrom::Start(self.position))?;
            size = self.file.read(&mut buffer[..size])?;
        }
        self.position += size as u64;
        Ok(size)
    }
}

impl Seek for IndexedFile {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        let position = match position {
            SeekFrom::Start(p) => i128::from(p),
            SeekFrom::End(p) => i128::from(self.length) + i128::from(p),
            SeekFrom::Current(p) => i128::from(self.position) + i128::from(p),
        };
        self.position =
            u64::try_from(position).map_err(|_| io::Error::other("invalid ZIP seek"))?;
        Ok(self.position)
    }
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
