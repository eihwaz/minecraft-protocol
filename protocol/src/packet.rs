use std::io::{Read, Write};

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

use crate::decoder::{Decoder, DecoderReadExt};
use crate::encoder::{Encoder, EncoderWriteExt};
use crate::error::{DecodeError, EncodeError};

#[derive(Debug)]
pub struct RawPacket {
    pub id: i32,
    pub data: Vec<u8>,
}

impl Encoder for RawPacket {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_var_i32(self.id)?;
        writer.write_all(&self.data)?;

        Ok(())
    }
}

impl Decoder for RawPacket {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let id = reader.read_var_i32()?;

        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        Ok(Self { id, data })
    }
}

#[derive(Debug)]
pub struct CompressedRawPacket {
    packet: RawPacket,
    threshold: i32,
}

impl Encoder for CompressedRawPacket {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        let mut packet_buf = Vec::new();
        self.packet.encode(&mut packet_buf)?;

        let data_len = packet_buf.len() as i32;
        let mut packet = Vec::new();
        if self.threshold >= 0 && data_len > self.threshold {
            packet.write_var_i32(data_len)?;
            let mut encoder = ZlibEncoder::new(&mut packet, Compression::default());
            encoder.write_all(&packet_buf)?;
            encoder.finish()?;
        } else {
            packet.write_var_i32(0)?;
            packet.write_all(&packet_buf)?;
        };

        writer.write_var_i32(packet.len() as i32)?;
        writer.write_all(&packet)?;

        Ok(())
    }
}

impl Decoder for CompressedRawPacket {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let packet_len = reader.read_var_i32()? as u64;
        let mut reader = reader.take(packet_len);

        let data_len = reader.read_var_i32()? as usize;
        let packet = if data_len == 0 {
            RawPacket::decode(&mut reader)?
        } else {
            let mut decompressed = Vec::with_capacity(data_len);
            ZlibDecoder::new(reader).read_to_end(&mut decompressed)?;

            if decompressed.len() != data_len {
                return Err(DecodeError::DecompressionError);
            }

            RawPacket::decode(&mut decompressed.as_slice())?
        };

        Ok(CompressedRawPacket {
            packet,
            threshold: Default::default(),
        })
    }
}

#[derive(Debug)]
pub struct UncompressedRawPacket {
    packet: RawPacket,
}

impl Encoder for UncompressedRawPacket {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        let mut packet_buf = Vec::new();
        self.packet.encode(&mut packet_buf)?;

        writer.write_var_i32(packet_buf.len() as i32)?;
        writer.write_all(&packet_buf)?;

        Ok(())
    }
}

impl Decoder for UncompressedRawPacket {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let len = reader.read_var_i32()? as u64;
        let packet = RawPacket::decode(&mut reader.take(len))?;

        Ok(Self { packet })
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

        let packet = UncompressedRawPacket {
            packet: RawPacket { id: 1, data },
        };

        let mut vec = Vec::new();
        packet.encode(&mut vec).unwrap();

        assert_eq!(vec, ping_request_packet_bytes());
    }

    #[test]
    fn test_uncompressed_packet_decode() {
        let vec = ping_request_packet_bytes();
        let packet = UncompressedRawPacket::decode(&mut vec.as_slice())
            .unwrap()
            .packet;

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

        let packet = CompressedRawPacket {
            packet: RawPacket { id: 1, data },
            threshold: 0,
        };

        let mut vec = Vec::new();
        packet.encode(&mut vec).unwrap();

        let packet = CompressedRawPacket::decode(&mut vec.as_slice())
            .unwrap()
            .packet;

        assert_eq!(packet.id, 1);
        assert_eq!(packet.data, PING_REQUEST_BYTES);
    }
}
