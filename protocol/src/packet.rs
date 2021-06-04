use std::io::{Cursor, Read, Write};

use crate::decoder::{Decoder, DecoderReadExt};
use crate::encoder::{Encoder, EncoderWriteExt};
use crate::error::{DecodeError, EncodeError};

#[derive(Debug)]
pub struct Packet {
    pub id: i32,
    pub data: Vec<u8>,
}

impl Encoder for Packet {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        let mut id_buf = Vec::new();
        id_buf.write_var_i32(self.id)?;

        writer.write_var_i32((id_buf.len() + self.data.len()) as i32)?;
        writer.write(&id_buf)?;
        writer.write(&self.data)?;

        Ok(())
    }
}

impl Decoder for Packet {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let length = reader.read_var_i32()?;

        let mut buf = vec![0; length as usize];
        reader.read_exact(&mut buf)?;

        let mut cursor = Cursor::new(&mut buf);
        let id = cursor.read_var_i32()?;
        let position = cursor.position() as usize;
        let data = buf.split_off(position);

        Ok(Self { id, data })
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
    fn test_packet_encode() {
        let ping_request = PingRequest {
            time: 1577735845610,
        };

        let mut data = Vec::new();
        ping_request.encode(&mut data).unwrap();

        let packet = Packet { id: 1, data };

        let mut vec = Vec::new();
        packet.encode(&mut vec).unwrap();

        assert_eq!(vec, ping_request_packet_bytes());
    }

    #[test]
    fn test_packet_decode() {
        let mut cursor = Cursor::new(ping_request_packet_bytes());
        let packet = Packet::decode(&mut cursor).unwrap();

        assert_eq!(packet.id, 1);
        assert_eq!(packet.data, PING_REQUEST_BYTES);
    }
}
