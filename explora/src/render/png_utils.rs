use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

/// In-memory PNG image, with RGBA format (8 bits per channel).
#[derive(Debug)]
pub struct PngImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    /// Bytes per channel
    pub channels: u8,
}

pub type Pixel = [u8; 4];

#[derive(Debug)]
pub enum PngImageError {
    IoError(std::io::Error),
    PngDecodingError(png::DecodingError),
    PngEncodingError(png::EncodingError),
    UnsupportedColorType(png::ColorType),
}

impl From<std::io::Error> for PngImageError {
    fn from(error: std::io::Error) -> Self {
        PngImageError::IoError(error)
    }
}

impl From<png::DecodingError> for PngImageError {
    fn from(error: png::DecodingError) -> Self {
        PngImageError::PngDecodingError(error)
    }
}

impl From<png::EncodingError> for PngImageError {
    fn from(error: png::EncodingError) -> Self {
        PngImageError::PngEncodingError(error)
    }
}

/// Writes the given buffer as a PNG image  using a Buffered Writer.
///
/// The image is assumed to be in RGBA format
pub fn write<P: AsRef<Path>>(
    path: P,
    buf: &[u8],
    width: u32,
    height: u32,
) -> Result<(), PngImageError> {
    let w = BufWriter::new(File::create(path)?);
    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Default);
    encoder.set_filter(png::FilterType::default());
    encoder.set_adaptive_filter(png::AdaptiveFilterType::default());
    let mut writer = encoder.write_header()?;
    writer.write_image_data(buf)?; // Save
    Ok(())
}

/// Reads a PNG image from the given path, using a Buffered Reader.
///
/// The image is assumed to be in RGBA format.
pub fn read<P: AsRef<Path>>(path: P) -> Result<PngImage, PngImageError> {
    let buffered_read = BufReader::new(File::open(path)?);
    let limits = png::Limits::default(); // 64 megabytes
    let mut decoder = png::Decoder::new_with_limits(buffered_read, limits);
    decoder.set_ignore_text_chunk(true); // We don't care about text
    decoder.set_transformations(png::Transformations::all()); // Apply transformations needed
    let mut reader = decoder.read_info()?;
    let mut image = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut image)?;
    let info = reader.info();

    let channels = match reader.output_color_type() {
        (png::ColorType::Rgba, _) => 4,
        (png::ColorType::Rgb, _) => 3,
        (color_type, _) => return Err(PngImageError::UnsupportedColorType(color_type)),
    };
    Ok(PngImage {
        width: info.width,
        height: info.height,
        pixels: image,
        channels,
    })
}
