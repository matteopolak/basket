use std::io::Read;

use crate::Error;

pub fn skip<R, const N: usize>(reader: &mut R, seq: &[u8; N]) -> Result<(), Error>
where
	R: Read,
{
	let mut buf = [0; N];

	reader.read_exact(&mut buf)?;

	if &buf != seq {
		return Err(Error::InvalidFormat);
	}

	Ok(())
}

pub fn until<R>(reader: &mut R, seq: &[u8]) -> Result<Vec<u8>, Error>
where
	R: Read,
{
	let mut i = 0;
	let mut extracted = Vec::with_capacity(seq.len());

	reader.take(seq.len() as u64).read_to_end(&mut extracted)?;

	while !extracted[i..].starts_with(seq) {
		i += 1;

		reader.take(1).read_to_end(&mut extracted)?;
	}

	// once it starts with seq, we need to remove it
	extracted.truncate(i);

	Ok(extracted)
}

pub fn http_version<R>(reader: &mut R) -> Result<(), Error>
where
	R: Read,
{
	let mut buf = [0; b"HTTP/1.1".len()];

	reader.read_exact(&mut buf)?;

	if &buf != b"HTTP/1.1" {
		return Err(Error::UnsupportedHttp);
	}

	Ok(())
}
