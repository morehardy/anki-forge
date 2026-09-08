//! Stored ZIP entries with known sizes and CRCs. Headers are final on their
//! first write, so the package fingerprint covers final bytes without rereads.
use std::io::{self, BufWriter, Write};

use super::pipelined_sha1::PipelinedSha1;

pub(crate) struct StreamZip<W: Write> {
    output: BufWriter<W>,
    hash: PipelinedSha1,
    position: u64,
    central: Vec<Vec<u8>>,
}

impl<W: Write> StreamZip<W> {
    pub(crate) fn new(output: W) -> Self {
        Self {
            output: BufWriter::with_capacity(128 * 1024, output),
            hash: PipelinedSha1::new(),
            position: 0,
            central: Vec::new(),
        }
    }

    pub(crate) fn bytes(&mut self, name: &str, bytes: &[u8]) -> anyhow::Result<()> {
        self.entry(name, bytes.len() as u64, crc32fast::hash(bytes), |output| {
            output.write_all(bytes)?;
            Ok(())
        })
    }

    pub(crate) fn entry(
        &mut self,
        name: &str,
        size: u64,
        crc: u32,
        write: impl FnOnce(&mut Self) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let name_len = u16::try_from(name.len())?;
        let offset = self.position;
        let large_size = size >= u32::MAX as u64;
        let large_offset = offset >= u32::MAX as u64;
        let version = if large_size { 45u16 } else { 10 };
        let flags: u16 = if name.is_ascii() { 0 } else { 0x800 };
        let size32 = if large_size { u32::MAX } else { size as u32 };
        let mut local = Vec::with_capacity(30 + name.len() + 20);
        u32le(&mut local, 0x04034b50);
        for value in [version, flags, 0, 0, 33] {
            u16le(&mut local, value);
        }
        for value in [crc, size32, size32] {
            u32le(&mut local, value);
        }
        u16le(&mut local, name_len);
        u16le(&mut local, if large_size { 20 } else { 0 });
        local.extend_from_slice(name.as_bytes());
        if large_size {
            u16le(&mut local, 1);
            u16le(&mut local, 16);
            u64le(&mut local, size);
            u64le(&mut local, size);
        }
        self.write_all(&local)?;
        let body_start = self.position;
        write(self)?;
        anyhow::ensure!(
            self.position - body_start == size,
            "ZIP entry {name} length changed while writing"
        );

        let central_version = if large_size || large_offset {
            45
        } else {
            version
        };
        let mut extra = Vec::new();
        if large_size {
            u64le(&mut extra, size);
            u64le(&mut extra, size);
        }
        if large_offset {
            u64le(&mut extra, offset);
        }
        let mut central = Vec::with_capacity(46 + name.len() + extra.len() + 4);
        u32le(&mut central, 0x02014b50);
        for value in [0x300 | central_version, central_version, flags, 0, 0, 33] {
            u16le(&mut central, value);
        }
        for value in [crc, size32, size32] {
            u32le(&mut central, value);
        }
        u16le(&mut central, name_len);
        u16le(
            &mut central,
            if extra.is_empty() {
                0
            } else {
                extra.len() as u16 + 4
            },
        );
        for _ in 0..3 {
            u16le(&mut central, 0);
        }
        u32le(&mut central, 0o100644 << 16);
        u32le(
            &mut central,
            if large_offset {
                u32::MAX
            } else {
                offset as u32
            },
        );
        central.extend_from_slice(name.as_bytes());
        if !extra.is_empty() {
            u16le(&mut central, 1);
            u16le(&mut central, extra.len() as u16);
            central.extend_from_slice(&extra);
        }
        self.central.push(central);
        Ok(())
    }

    pub(crate) fn finish(mut self) -> io::Result<(W, String)> {
        let start = self.position;
        let central = std::mem::take(&mut self.central);
        let count = central.len() as u64;
        for entry in central {
            self.write_all(&entry)?;
        }
        let size = self.position - start;
        let mut end = Vec::new();
        if count >= u16::MAX as u64 || size >= u32::MAX as u64 || start >= u32::MAX as u64 {
            let offset = self.position;
            u32le(&mut end, 0x06064b50);
            u64le(&mut end, 44);
            u16le(&mut end, 0x32d);
            u16le(&mut end, 45);
            u32le(&mut end, 0);
            u32le(&mut end, 0);
            for value in [count, count, size, start] {
                u64le(&mut end, value);
            }
            u32le(&mut end, 0x07064b50);
            u32le(&mut end, 0);
            u64le(&mut end, offset);
            u32le(&mut end, 1);
        }
        u32le(&mut end, 0x06054b50);
        u16le(&mut end, 0);
        u16le(&mut end, 0);
        u16le(&mut end, count.min(u16::MAX as u64) as u16);
        u16le(&mut end, count.min(u16::MAX as u64) as u16);
        u32le(&mut end, size.min(u32::MAX as u64) as u32);
        u32le(&mut end, start.min(u32::MAX as u64) as u32);
        u16le(&mut end, 0);
        self.write_all(&end)?;
        self.output.flush()?;
        let fingerprint = format!("package:{}", self.hash.finish()?);
        Ok((
            self.output
                .into_inner()
                .map_err(|error| error.into_error())?,
            fingerprint,
        ))
    }
}

impl<W: Write> Write for StreamZip<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let written = self.output.write(bytes)?;
        self.hash.update(&bytes[..written])?;
        self.position = self
            .position
            .checked_add(written as u64)
            .ok_or_else(|| io::Error::other("ZIP position overflow"))?;
        Ok(written)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

fn u16le(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}
fn u32le(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}
fn u64le(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha1::{Digest, Sha1};
    use std::io::{Cursor, Read};

    struct ShortWriter {
        bytes: Vec<u8>,
        fail_after: Option<usize>,
    }

    impl Write for ShortWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let remaining = self.fail_after.unwrap_or(usize::MAX) - self.bytes.len();
            if remaining == 0 {
                return Err(io::Error::other("injected write failure"));
            }
            let count = bytes.len().min(4093).min(remaining);
            self.bytes.extend_from_slice(&bytes[..count]);
            Ok(count)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn large_zip_keeps_exact_bytes_and_fingerprint_with_short_writes() {
        let payload: Vec<_> = (0..2 * 1024 * 1024 + 17).map(|i| (i * 37) as u8).collect();
        let entries = [
            ("large", payload.as_slice()),
            ("empty", &[]),
            ("tail", b"tail".as_slice()),
        ];
        let mut stream = StreamZip::new(ShortWriter {
            bytes: Vec::new(),
            fail_after: None,
        });
        let mut reference = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            stream
                .entry(name, bytes.len() as u64, crc32fast::hash(bytes), |writer| {
                    for chunk in bytes.chunks(65537) {
                        writer.write_all(chunk)?;
                    }
                    Ok(())
                })
                .unwrap();
            reference
                .start_file(
                    name,
                    zip::write::SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Stored),
                )
                .unwrap();
            reference.write_all(bytes).unwrap();
        }
        let (written, fingerprint) = stream.finish().unwrap();
        assert_eq!(written.bytes, reference.finish().unwrap().into_inner());
        assert_eq!(
            fingerprint,
            format!("package:{}", hex::encode(Sha1::digest(&written.bytes)))
        );
    }

    #[test]
    fn large_zip_propagates_midstream_output_errors() {
        let bytes = vec![41; 2 * 1024 * 1024];
        let mut stream = StreamZip::new(ShortWriter {
            bytes: Vec::new(),
            fail_after: Some(1024 * 1024 + 7),
        });
        let error = stream
            .entry(
                "large",
                bytes.len() as u64,
                crc32fast::hash(&bytes),
                |writer| {
                    writer.write_all(&bytes)?;
                    Ok(())
                },
            )
            .unwrap_err();
        assert!(error.to_string().contains("injected write failure"));
        drop(stream);
    }

    #[test]
    fn final_bytes_match_the_existing_zip_writer_and_fingerprint() {
        let entries = [("meta", b"abc".as_slice()), ("0", b"media"), ("media", b"")];
        let mut stream = StreamZip::new(Vec::new());
        let mut old = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            stream.bytes(name, bytes).unwrap();
            old.start_file(
                name,
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored),
            )
            .unwrap();
            old.write_all(bytes).unwrap();
        }
        let (bytes, fingerprint) = stream.finish().unwrap();
        assert_eq!(bytes, old.finish().unwrap().into_inner());
        assert_eq!(
            fingerprint,
            format!("package:{}", hex::encode(Sha1::digest(&bytes)))
        );
    }

    #[test]
    fn zip64_entry_count_is_readable_by_independent_zip_reader() {
        let mut writer = StreamZip::new(Vec::new());
        for index in 0..=u16::MAX {
            writer.bytes(&index.to_string(), b"").unwrap();
        }
        let (bytes, _) = writer.finish().unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert_eq!(archive.len(), 65536);
        let mut last = Vec::new();
        archive
            .by_name("65535")
            .unwrap()
            .read_to_end(&mut last)
            .unwrap();
        assert!(last.is_empty());
    }

    #[test]
    fn wrong_length_and_destination_errors_abort_writing() {
        let mut stream = StreamZip::new(Vec::new());
        assert!(stream
            .entry("media", 2, 0, |writer| {
                writer.write_all(b"x")?;
                Ok(())
            })
            .is_err());
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("injected output failure"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut stream = StreamZip::new(Broken);
        stream.bytes("meta", b"test").unwrap();
        assert!(stream.finish().is_err());
    }
}
