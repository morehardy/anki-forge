#![cfg(feature = "internal-tools")]

use std::io::{self, Write};

use anki_forge::deck::{Deck, MediaSource, Package};

struct ShortWriter {
    bytes: Vec<u8>,
    largest_request: usize,
    interrupted: bool,
}

impl Write for ShortWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.largest_request = self.largest_request.max(bytes.len());
        if !self.interrupted {
            self.interrupted = true;
            return Err(io::ErrorKind::Interrupted.into());
        }
        let count = bytes.len().min(113);
        self.bytes.extend_from_slice(&bytes[..count]);
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        panic!("write_to must leave flushing a caller-owned writer to its owner")
    }
}

fn short_writer() -> ShortWriter {
    ShortWriter {
        bytes: Vec::new(),
        largest_request: 0,
        interrupted: false,
    }
}

fn media_deck(root: &std::path::Path) -> Deck {
    let mut state = 0x456a_9b17_u32;
    let bytes: Vec<u8> = (0..256 * 1024)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as u8
        })
        .collect();
    let path = root.join("payload.bin");
    std::fs::write(&path, bytes).unwrap();
    let mut deck = Deck::builder("Writer").stable_id("writer").build();
    let media = deck.media().add(MediaSource::from_file(path)).unwrap();
    deck.basic()
        .note("front", format!("<object data='{}'>", media.name()))
        .stable_id("note")
        .add()
        .unwrap();
    deck
}

#[test]
fn exports_use_bounded_writes_and_handle_short_writes_and_interruptions() {
    let root = tempfile::tempdir().unwrap();
    let deck = media_deck(root.path());
    let package = Package::single(deck.clone());
    let expected = deck.to_apkg_bytes().unwrap();
    assert!(
        expected.len() > 128 * 1024,
        "fixture must exceed the copy buffer"
    );
    let mut writer = short_writer();
    deck.write_to(&mut writer).unwrap();
    assert_eq!(writer.bytes, expected);
    assert!(
        writer.largest_request <= 64 * 1024,
        "whole-package write of {} bytes",
        writer.largest_request
    );

    let expected_package = package.to_apkg_bytes().unwrap();
    let mut writer = short_writer();
    package.write_to(&mut writer).unwrap();
    assert_eq!(writer.bytes, expected_package);
    assert!(writer.largest_request <= 64 * 1024);

    // A completed or failed export must leave the editable source reusable.
    let mut repeat = Vec::new();
    deck.write_to(&mut repeat).unwrap();
    assert_eq!(repeat, expected);
}

struct FailingWriter {
    accepted: usize,
    zero: bool,
}

impl Write for FailingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.accepted == 127 {
            return if self.zero {
                Ok(0)
            } else {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "sink failed"))
            };
        }
        let count = bytes.len().min(127 - self.accepted);
        self.accepted += count;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn sink_failures_are_propagated_and_a_later_export_succeeds() {
    let root = tempfile::tempdir().unwrap();
    let deck = media_deck(root.path());
    let package = Package::single(deck.clone());
    for zero in [false, true] {
        for package_path in [false, true] {
            let mut writer = FailingWriter { accepted: 0, zero };
            let error = if package_path {
                package.write_to(&mut writer)
            } else {
                deck.write_to(&mut writer)
            }
            .unwrap_err();
            assert_eq!(
                error.downcast_ref::<io::Error>().unwrap().kind(),
                if zero {
                    io::ErrorKind::WriteZero
                } else {
                    io::ErrorKind::BrokenPipe
                }
            );
            assert_eq!(writer.accepted, 127);
        }
    }
    let mut written = Vec::new();
    deck.write_to(&mut written).unwrap();
    assert_eq!(written, deck.to_apkg_bytes().unwrap());
}

#[test]
fn build_failure_does_not_touch_the_writer_or_existing_destination() {
    let root = tempfile::tempdir().unwrap();
    let deck = media_deck(root.path());
    let package = Package::single(deck.clone());
    std::fs::remove_file(root.path().join("payload.bin")).unwrap();
    let mut output = Vec::new();
    assert!(deck.write_to(&mut output).is_err());
    assert!(package.write_to(&mut output).is_err());
    assert!(output.is_empty());
    let destination = root.path().join("existing.apkg");
    std::fs::write(&destination, b"existing output").unwrap();
    assert!(package.write_apkg(&destination).is_err());
    assert_eq!(std::fs::read(destination).unwrap(), b"existing output");
}
