//! One streaming read path for both ZIP output and nested zstd content.
use std::cell::Cell;
use std::io::{self, BufRead, BufReader, Cursor, Read, Write};
use std::path::Path;

use anyhow::{ensure, Result};
use zip::ZipArchive;

use super::inspect_limits::{check, InspectLimits};

const BUFFER_BYTES: usize = 64 * 1024;

pub(crate) struct ApkgReader<'a> {
    archive: ZipArchive<super::apkg_index::IndexedFile>,
    pub limits: &'a InspectLimits,
    zip_used: Cell<u64>,
    decoded_used: Cell<u64>,
}

impl<'a> ApkgReader<'a> {
    pub fn open(path: &Path, limits: &'a InspectLimits) -> Result<Self> {
        let archive = super::apkg_index::open(path, limits)?;
        Ok(Self {
            archive,
            limits,
            zip_used: Cell::new(0),
            decoded_used: Cell::new(0),
        })
    }

    pub fn contains(&self, name: &str) -> bool {
        self.archive.index_for_name(name).is_some()
    }

    pub fn bytes(
        &mut self,
        name: &str,
        compressed: bool,
        resource: &'static str,
        limit: u64,
    ) -> Result<Option<Vec<u8>>> {
        let mut bytes = Vec::new();
        Ok(self
            .copy(name, compressed, resource, limit, &mut bytes)?
            .map(|_| bytes))
    }

    pub fn copy(
        &mut self,
        name: &str,
        compressed: bool,
        resource: &'static str,
        limit: u64,
        sink: &mut impl Write,
    ) -> Result<Option<u64>> {
        let entry = match self.archive.by_name(name) {
            Ok(entry) => entry,
            Err(zip::result::ZipError::FileNotFound) => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        check(
            "zip_entry_bytes",
            Some(name),
            self.limits.max_zip_entry_bytes,
            entry.size(),
        )?;
        let reader = ZipBudgetReader {
            inner: entry,
            name,
            used: 0,
            total: &self.zip_used,
            limits: self.limits,
        };
        let mut reader = BufReader::with_capacity(BUFFER_BYTES, reader);
        let mut output = DecodedSink {
            sink,
            name,
            resource,
            limit,
            used: 0,
            total: &self.decoded_used,
            total_limit: self.limits.max_decoded_total_bytes,
        };
        if compressed {
            decode_frames(&mut reader, &mut output, self.limits.max_zstd_window_bytes)?;
        } else {
            output.copy_from(&mut reader)?;
        }
        Ok(Some(output.used))
    }
}

struct ZipBudgetReader<'a, R> {
    inner: R,
    name: &'a str,
    used: u64,
    total: &'a Cell<u64>,
    limits: &'a InspectLimits,
}

impl<R: Read> Read for ZipBudgetReader<'_, R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }
        let remaining = (self.limits.max_zip_entry_bytes - self.used)
            .min(self.limits.max_zip_total_bytes - self.total.get());
        let size = out
            .len()
            .min(remaining.saturating_add(1).min(usize::MAX as u64) as usize);
        let count = self.inner.read(&mut out[..size])?;
        let entry_used = self.used.saturating_add(count as u64);
        let total_used = self.total.get().saturating_add(count as u64);
        check(
            "zip_entry_bytes",
            Some(self.name),
            self.limits.max_zip_entry_bytes,
            entry_used,
        )
        .map_err(io::Error::other)?;
        check(
            "zip_total_bytes",
            Some(self.name),
            self.limits.max_zip_total_bytes,
            total_used,
        )
        .map_err(io::Error::other)?;
        self.used = entry_used;
        self.total.set(total_used);
        Ok(count)
    }
}

struct DecodedSink<'a, W> {
    sink: &'a mut W,
    name: &'a str,
    resource: &'static str,
    limit: u64,
    used: u64,
    total: &'a Cell<u64>,
    total_limit: u64,
}

impl<W: Write> DecodedSink<'_, W> {
    fn copy_from(&mut self, reader: &mut impl Read) -> Result<()> {
        let mut buffer = [0; BUFFER_BYTES];
        loop {
            let remaining = (self.limit - self.used).min(self.total_limit - self.total.get());
            let size = buffer
                .len()
                .min(remaining.saturating_add(1).min(usize::MAX as u64) as usize);
            let count = match reader.read(&mut buffer[..size]) {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                result => result?,
            };
            if count == 0 {
                return Ok(());
            }
            let used = self.used.saturating_add(count as u64);
            let total = self.total.get().saturating_add(count as u64);
            check(self.resource, Some(self.name), self.limit, used)?;
            check(
                "decoded_total_bytes",
                Some(self.name),
                self.total_limit,
                total,
            )?;
            self.used = used;
            self.total.set(total);
            self.sink.write_all(&buffer[..count])?;
        }
    }
}

// Read a bounded frame header before constructing a decoder. This preserves a
// typed window-limit error and prevents single-segment frames from allocating
// their declared content size. Each concatenated frame shares the same counters.
fn decode_frames(
    reader: &mut impl BufRead,
    output: &mut DecodedSink<'_, impl Write>,
    window_limit: u64,
) -> Result<()> {
    let mut saw_frame = false;
    while !reader.fill_buf()?.is_empty() {
        let mut header = [0u8; 18];
        reader.read_exact(&mut header[..4])?;
        let magic = u32::from_le_bytes(header[..4].try_into().unwrap());
        if magic & 0xffff_fff0 == 0x184d_2a50 {
            // Skippable frame payload is charged to the ZIP budget, not decoded content.
            reader.read_exact(&mut header[4..8])?;
            let size = u32::from_le_bytes(header[4..8].try_into().unwrap()) as u64;
            let copied = io::copy(&mut reader.take(size), &mut io::sink())?;
            ensure!(copied == size, "truncated skippable zstd frame");
            saw_frame = true;
            continue;
        }
        ensure!(magic == 0xfd2f_b528, "invalid zstd frame magic");
        reader.read_exact(&mut header[4..5])?;
        let descriptor = header[4];
        ensure!(descriptor & 0x18 == 0, "reserved zstd frame header bits");
        let single = descriptor & 0x20 != 0;
        let dictionary_bytes = [0, 1, 2, 4][(descriptor & 3) as usize];
        let content_bytes = match descriptor >> 6 {
            0 => usize::from(single),
            1 => 2,
            2 => 4,
            _ => 8,
        };
        let content_start = 5 + usize::from(!single) + dictionary_bytes;
        let header_size = content_start + content_bytes;
        reader.read_exact(&mut header[5..header_size])?;
        let window = if single {
            let mut bytes = [0; 8];
            bytes[..content_bytes].copy_from_slice(&header[content_start..header_size]);
            u64::from_le_bytes(bytes) + if content_bytes == 2 { 256 } else { 0 }
        } else {
            let base = 1u64 << (10 + (header[5] >> 3));
            base + (base / 8) * u64::from(header[5] & 7)
        };
        check("zstd_window_bytes", Some(output.name), window_limit, window)?;
        let replay = Cursor::new(&header[..header_size]).chain(&mut *reader);
        let mut decoder = zstd::stream::read::Decoder::with_buffer(replay)?.single_frame();
        // The explicit header check handles exact, non-power-of-two limits;
        // the decoder setting is a second guard against oversized windows.
        let window_log = (64 - window_limit.max(1).saturating_sub(1).leading_zeros()).clamp(10, 31);
        decoder.window_log_max(window_log)?;
        output.copy_from(&mut decoder)?;
        saw_frame = true;
    }
    ensure!(saw_frame, "empty zstd stream");
    Ok(())
}
