use std::io::{self, Read, Write};

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use minecraft_protocol_derive::{Decoder, Encoder};

use crate::decoder::{Decoder, DecoderReadExt};
use crate::encoder::{Encoder, EncoderWriteExt};
use crate::error::{DecodeError, EncodeError};

#[derive(Debug, Clone)]
pub struct Packet {
    pub id: i32,
    pub data: Vec<u8>,
}

impl Packet {
    pub fn encode<W: Write>(
        self,
        writer: &mut W,
        compression_threshold: Option<i32>,
    ) -> Result<(), EncodeError> {
        let mut buf = Vec::new();
        let packet = RawPacket {
            id: self.id,
            data: self.data,
        };
        if let Some(threshold) = compression_threshold {
            CompressedRawPacket { packet, threshold }.encode(&mut buf)?;
        } else {
            packet.encode(&mut buf)?;
        }

        writer.write_var_i32(buf.len() as i32)?;
        writer.write_all(&buf)?;

        Ok(())
    }

    pub fn decode<R: Read>(reader: &mut R, compressed: bool) -> Result<Packet, DecodeError> {
        let len = match reader.read_var_i32() {
            Ok(len) => len as usize,
            Err(DecodeError::IoError { io_error })
                if io_error.kind() == io::ErrorKind::UnexpectedEof =>
            {
                return Err(DecodeError::Incomplete { bytes_needed: 1 })
            }
            Err(err) => return Err(err.into()),
        };

        let mut buf = Vec::with_capacity(len);
        let bytes_read = reader.take(len as u64).read_to_end(&mut buf)?;

        if bytes_read != len {
            return Err(DecodeError::Incomplete {
                bytes_needed: len - bytes_read,
            });
        }

        let RawPacket { id, data } = if compressed {
            CompressedRawPacket::decode(&mut buf.as_slice())?
        } else {
            RawPacket::decode(&mut buf.as_slice())?
        };

        Ok(Self { id, data })
    }
}

#[derive(Debug, Clone, Encoder, Decoder)]
struct RawPacket {
    #[data_type(with = "var_int")]
    pub id: i32,
    #[data_type(with = "rest")]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
struct CompressedRawPacket {
    packet: RawPacket,
    threshold: i32,
}

impl Encoder for CompressedRawPacket {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        let mut packet_buf = Vec::new();
        self.packet.encode(&mut packet_buf)?;

        let data_len = packet_buf.len() as i32;
        if self.threshold >= 0 && data_len > self.threshold {
            writer.write_var_i32(data_len)?;
            let mut encoder = ZlibEncoder::new(writer, Compression::default());
            encoder.write_all(&packet_buf)?;
            encoder.finish()?;
        } else {
            writer.write_var_i32(0)?;
            writer.write_all(&packet_buf)?;
        };

        Ok(())
    }
}

impl Decoder for CompressedRawPacket {
    type Output = RawPacket;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let data_len = reader.read_var_i32()? as usize;
        let packet = if data_len == 0 {
            RawPacket::decode(reader)?
        } else {
            let mut decompressed = Vec::with_capacity(data_len);
            ZlibDecoder::new(reader).read_to_end(&mut decompressed)?;

            if decompressed.len() != data_len {
                return Err(DecodeError::DecompressionError);
            }

            RawPacket::decode(&mut decompressed.as_slice())?
        };

        Ok(packet)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;

    use crate::version::v1_14_4::status::*;

    const PING_REQUEST_BYTES: &'static [u8] =
        include_bytes!("../test/packet/status/ping_request.dat");

    fn ping_request_packet_bytes() -> Vec<u8> {
        let len = (1 + PING_REQUEST_BYTES.len()).try_into().unwrap();
        let mut vec = vec![len, 1];
        vec.extend(PING_REQUEST_BYTES);
        vec
    }

    #[test]
    fn test_uncompressed_packet_encode() {
        let ping_request = PingRequest {
            time: 1577735845610,
        };

        let mut data = Vec::new();
        ping_request.encode(&mut data).unwrap();

        let packet = Packet { id: 1, data };

        let mut vec = Vec::new();
        packet.encode(&mut vec, None).unwrap();

        assert_eq!(vec, ping_request_packet_bytes());
    }

    #[test]
    fn test_uncompressed_packet_decode() {
        let vec = ping_request_packet_bytes();
        let packet = Packet::decode(&mut vec.as_slice(), false).unwrap();

        assert_eq!(packet.id, 1);
        assert_eq!(packet.data, PING_REQUEST_BYTES);
    }

    #[test]
    fn test_compressed_packet_encode_decode() {
        let ping_request = PingRequest {
            time: 1577735845610,
        };

        let mut data = Vec::new();
        ping_request.encode(&mut data).unwrap();

        let packet = Packet { id: 1, data };

        let mut vec = Vec::new();
        packet.encode(&mut vec, Some(0)).unwrap();

        let packet = Packet::decode(&mut vec.as_slice(), true).unwrap();

        assert_eq!(packet.id, 1);
        assert_eq!(packet.data, PING_REQUEST_BYTES);
    }
}
