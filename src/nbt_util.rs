//! Pretty much everything in this module was ripped from the `raw` module in the `hematite-nbt` crate.

use byteorder::{BigEndian, ReadBytesExt};
use flate2::read::GzDecoder;

/// Extracts an `Blob` object from an `io::Read` source that is
/// compressed using the Gzip format.
pub fn from_gzip_reader<R>(src: &mut R) -> nbt::Result<(String, nbt::Value)>
where
    R: std::io::Read,
{
    // Reads the gzip header, and fails if it is incorrect.
    let mut data = GzDecoder::new(src);
    from_reader(&mut data)
}

#[inline]
pub fn from_reader<R>(src: &mut R) -> nbt::Result<(String, nbt::Value)>
where
    R: std::io::Read,
{
    // Try to read the first tag (should be a compound tag)
    let tag = src.read_u8()?;
    // We must at least consume this title
    let title = match tag {
        0x00 => "".to_string(),
        _ => read_bare_string(src)?,
    };

    // Although it would be possible to read NBT format files composed of
    // arbitrary objects using the current API, by convention all files
    // have a top-level Compound.
    if tag != 0x0a {
        return Err(nbt::Error::NoRootCompound);
    }
    let content = nbt::Value::from_reader(tag, src)?;
    match content {
        val @ nbt::Value::Compound(_) => Ok((title, val)),
        _ => Err(nbt::Error::NoRootCompound),
    }
}

#[inline]
pub fn read_bare_string<R>(src: &mut R) -> nbt::Result<String>
where
    R: std::io::Read,
{
    let len = src.read_u16::<BigEndian>()? as usize;

    if len == 0 {
        return Ok("".to_string());
    }

    let mut bytes = vec![0; len];
    let mut n_read = 0usize;
    while n_read < bytes.len() {
        match src.read(&mut bytes[n_read..])? {
            0 => return Err(nbt::Error::IncompleteNbtValue),
            n => n_read += n,
        }
    }

    let decoded = cesu8::from_java_cesu8(&bytes)?;
    Ok(decoded.into_owned())
}
