use std::{
    collections::{BTreeMap, BTreeSet},
    io::{BufReader, Cursor},
};

use super::{ImageOcclusionError as Error, ImageOcclusionErrorKind as Kind};
use crate::{Media, Note};

/// How inactive masks appear while answering an image-occlusion card.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OcclusionMode {
    /// Cover every mask while asking for one. This is the default.
    #[default]
    HideAllGuessOne,
    /// Cover only the mask currently being asked about.
    HideOneGuessOne,
}

/// One rectangle in displayed image pixels, identified by a stable key.
///
/// The key is independent of position and declaration order. Coordinates refer
/// to the EXIF-oriented image. Validation occurs when the IO builder is finished.
#[derive(Debug, Clone, PartialEq)]
#[must_use = "add this mask to an ImageOcclusionBuilder"]
pub struct Mask {
    pub(crate) key: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Mask {
    /// Declares a rectangle with a stable key and pixel coordinates. Integers and
    /// floating-point inputs are accepted; values must be finite and in bounds.
    pub fn rect(
        key: impl Into<String>,
        x: impl Into<f64>,
        y: impl Into<f64>,
        width: impl Into<f64>,
        height: impl Into<f64>,
    ) -> Self {
        Self {
            key: key.into(),
            x: x.into(),
            y: y.into(),
            width: width.into(),
            height: height.into(),
        }
    }

    /// Returns the stable identity used to retain this mask's card on updates.
    pub fn key(&self) -> &str {
        &self.key
    }
}

/// A local image-occlusion declaration that owns its image and mask definitions.
/// Finishing it decodes the image and validates all coordinates before returning a Note.
#[derive(Debug, Clone)]
#[must_use = "call .build() to validate the image and masks and obtain a Note"]
pub struct ImageOcclusionBuilder {
    image: Media,
    masks: Vec<Mask>,
    mode: OcclusionMode,
}

impl Note {
    /// Begins an image-occlusion note using an owned snapshot. Add masks and call
    /// `.build()`, then use normal Note methods for `header`, `back_extra`,
    /// `comments`, deck and tags. The generated `image` and `occlusion` fields
    /// cannot be overridden.
    pub fn image_occlusion(image: Media) -> ImageOcclusionBuilder {
        ImageOcclusionBuilder {
            image,
            masks: Vec::new(),
            mode: OcclusionMode::default(),
        }
    }
}

impl ImageOcclusionBuilder {
    /// Adds a rectangle without assigning its eventual Anki card ordinal.
    pub fn mask(mut self, mask: Mask) -> Self {
        self.masks.push(mask);
        self
    }

    /// Sets how non-current masks are displayed.
    pub fn mode(mut self, mode: OcclusionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Fully decodes PNG, JPEG, GIF, WebP or BMP content and validates each mask.
    /// Coordinates use the image after EXIF rotation/reflection. When orientation
    /// changes the image, a normalized PNG snapshot is retained so rendering and
    /// validation use exactly the same pixels.
    ///
    /// The decoded pixel buffer is limited to 256 MiB, independently of the
    /// compressed media import budget. At most 500 distinct mask keys are allowed,
    /// matching Anki's supported cloze ordinals. Empty or invalid declarations
    /// return a structured error; dropping a builder does not add any note.
    pub fn build(self) -> Result<Note, Error> {
        if self.masks.is_empty() {
            return Err(Error::new(
                Kind::InvalidMask,
                "NOTE.IO_MASKS_EMPTY",
                "add at least one mask",
            ));
        }
        if self.masks.len() > 500 {
            return Err(Error::new(
                Kind::ResourceLimit,
                "NOTE.IO_ORDINAL_EXHAUSTED",
                "image occlusion supports at most 500 cloze ordinals; split this note",
            ));
        }
        let mut keys = BTreeSet::new();
        for mask in &self.masks {
            if mask.key.trim().is_empty()
                || mask.key.trim() != mask.key
                || mask.key.chars().any(char::is_control)
            {
                return Err(Error::new(Kind::InvalidMask, "NOTE.IO_MASK_KEY_INVALID", "mask key must be nonempty without surrounding whitespace or control characters"));
            }
            if !keys.insert(&mask.key) {
                return Err(Error::new(
                    Kind::InvalidMask,
                    "NOTE.IO_MASK_KEY_DUPLICATE",
                    format!("mask key {:?} occurs more than once", mask.key),
                ));
            }
        }
        let (image, width, height) = oriented_image(self.image)?;
        for mask in &self.masks {
            if ![mask.x, mask.y, mask.width, mask.height]
                .iter()
                .all(|value| value.is_finite())
                || mask.x < 0.0
                || mask.y < 0.0
                || mask.width <= 0.0
                || mask.height <= 0.0
                || mask.x + mask.width > f64::from(width)
                || mask.y + mask.height > f64::from(height)
            {
                return Err(Error::new(Kind::InvalidMask, "NOTE.IO_RECT_INVALID", format!("mask {:?} must have positive dimensions and fit in the displayed {width} × {height} image", mask.key)));
            }
        }
        let mut note = crate::schema::stock::image_occlusion().note();
        note.occlusion = Some(Occlusion {
            image,
            masks: self.masks,
            mode: self.mode,
            width,
            height,
        });
        Ok(note)
    }
}

/// Structured IO content survives until the build's identity assignments are known.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Occlusion {
    pub(crate) image: Media,
    pub(crate) masks: Vec<Mask>,
    mode: OcclusionMode,
    width: u32,
    height: u32,
}

impl Occlusion {
    pub(crate) fn render(&self, ordinals: &BTreeMap<String, u16>) -> String {
        use std::fmt::Write;
        let mut output = String::new();
        for mask in &self.masks {
            let ordinal = ordinals[&mask.key];
            let inactive = if self.mode == OcclusionMode::HideAllGuessOne {
                ":oi=1"
            } else {
                ""
            };
            write!(&mut output,
                "{{{{c{ordinal}::image-occlusion:rect:left={}:top={}:width={}:height={}{inactive}}}}}<br>",
                mask.x / f64::from(self.width), mask.y / f64::from(self.height),
                mask.width / f64::from(self.width), mask.height / f64::from(self.height),
            ).expect("write to String");
        }
        output
    }
}

fn oriented_image(media: Media) -> Result<(Media, u32, u32), Error> {
    use image::ImageDecoder;
    const MAX_DECODED_BYTES: u64 = 256 << 20;
    let (mut pixels, orientation) = {
        let source = media.snapshot.reader().map_err(media_error)?;
        let mut reader = image::ImageReader::new(BufReader::new(source))
            .with_guessed_format()
            .map_err(|cause| {
                Error::new(
                    Kind::Io,
                    "NOTE.IO_IMAGE_READ_FAILED",
                    "read owned image format",
                )
                .caused_by(cause)
            })?;
        let mut limits = image::Limits::default();
        limits.max_alloc = Some(MAX_DECODED_BYTES);
        reader.limits(limits.clone());
        let mut decoder = reader.into_decoder().map_err(image_error)?;
        // Reserve the final pixel buffer as well as the decoder's workspace,
        // following ImageReader::decode while retaining orientation metadata.
        limits.reserve(decoder.total_bytes()).map_err(image_error)?;
        decoder.set_limits(limits).map_err(image_error)?;
        let orientation = decoder.orientation().map_err(image_error)?;
        (
            image::DynamicImage::from_decoder(decoder).map_err(image_error)?,
            orientation,
        )
    };
    pixels.apply_orientation(orientation);
    let (width, height) = (pixels.width(), pixels.height());
    if width == 0 || height == 0 {
        return Err(Error::new(
            Kind::InvalidImage,
            "NOTE.IO_IMAGE_INVALID",
            "image must have positive dimensions",
        ));
    }
    let image = if orientation == image::metadata::Orientation::NoTransforms {
        media
    } else {
        let mut bytes = Cursor::new(Vec::new());
        pixels
            .write_to(&mut bytes, image::ImageFormat::Png)
            .map_err(image_error)?;
        Media::bytes(bytes.into_inner(), "image/png").map_err(media_error)?
    };
    Ok((image, width, height))
}

fn image_error(cause: image::ImageError) -> Error {
    let (kind, code) = match &cause {
        image::ImageError::Limits(_) => (Kind::ResourceLimit, "NOTE.IO_IMAGE_RESOURCE_LIMIT"),
        image::ImageError::IoError(_) => (Kind::Io, "NOTE.IO_IMAGE_READ_FAILED"),
        _ => (Kind::InvalidImage, "NOTE.IO_IMAGE_INVALID"),
    };
    Error::new(kind, code, "decode image-occlusion content").caused_by(cause)
}

fn media_error(cause: crate::media::MediaError) -> Error {
    let (kind, code) = if cause.kind() == crate::media::MediaErrorKind::ResourceLimit {
        (Kind::ResourceLimit, "NOTE.IO_IMAGE_RESOURCE_LIMIT")
    } else {
        (Kind::Io, "NOTE.IO_IMAGE_READ_FAILED")
    };
    Error::new(kind, code, "access owned image snapshot").caused_by(cause)
}
