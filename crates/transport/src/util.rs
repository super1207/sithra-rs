use bytes::BytesMut;
use tokio::process::Child;
use tokio_util::codec::Framed;

use crate::{
    datapack::{DataPackCodec, RawDataPackCodec},
    peer::Peer,
};

/// Connects to a child process and returns a framed transport.
///
/// # Errors
/// Returns an error if the child process fails to start or if the framed
/// transport fails to be created.
#[allow(clippy::result_large_err)]
pub fn connect(child: Child) -> Result<Framed<Peer, DataPackCodec>, Child> {
    let peer = Peer::from_child(child)?;
    let codec = DataPackCodec::new();
    Ok(Framed::new(peer, codec))
}

/// Connects to a child process and returns a framed transport using raw data
/// packing.
///
/// # Errors
/// Returns an error if the child process fails to start or if the framed
/// transport fails to be created.
#[allow(clippy::result_large_err)]
pub fn raw_connect(child: Child) -> Result<Framed<Peer, RawDataPackCodec>, Child> {
    let peer = Peer::from_child(child)?;
    let codec = RawDataPackCodec::new();
    Ok(Framed::new(peer, codec))
}

/// Creates a framed transport using standard input and output.
#[must_use]
pub fn stdio() -> Framed<Peer, DataPackCodec> {
    let peer = Peer::new();
    let codec = DataPackCodec::new();
    Framed::new(peer, codec)
}

/// Creates a framed transport using standard input and output.
#[must_use]
pub fn raw_stdio() -> Framed<Peer, RawDataPackCodec> {
    let peer = Peer::new();
    let codec = RawDataPackCodec::new();
    Framed::new(peer, codec)
}

pub fn get_chunk(src: &mut BytesMut) -> Option<BytesMut> {
    if src.is_empty() {
        None
    } else if src.len() < 1024 {
        Some(src.split_to(src.len()))
    } else {
        Some(src.split_to(1024))
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;

    #[test]
    fn test_get_chunk() {
        // Test with an empty buffer
        let mut src = BytesMut::new();
        let chunk = get_chunk(&mut src);
        assert_eq!(chunk, None);
        assert_eq!(src, BytesMut::new());

        // Test with a buffer smaller than the chunk size
        let mut src = BytesMut::from("Hello, world!");
        let chunk = get_chunk(&mut src);
        assert_eq!(chunk, Some(BytesMut::from("Hello, world!")));
        assert_eq!(src, BytesMut::new());

        // Test with a buffer larger than the chunk size
        let mut src = BytesMut::from(Bytes::from(vec![10u8; 2048]));
        let chunk = get_chunk(&mut src);
        assert_eq!(chunk, Some(BytesMut::from(Bytes::from(vec![10u8; 1024]))));
        assert_eq!(src, BytesMut::from(Bytes::from(vec![10u8; 1024])));

        // Test with a buffer exactly the chunk size
        let mut src = BytesMut::from(Bytes::from(vec![10u8; 1024]));
        let chunk = get_chunk(&mut src);
        assert_eq!(chunk, Some(BytesMut::from(Bytes::from(vec![10u8; 1024]))));
        assert_eq!(src, BytesMut::new());
    }
}
