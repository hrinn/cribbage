// Used to send messages to the client
// Async TcpStream
use crate::frame::Frame;
use bufstream::BufStream;
use bytes::{Buf, BytesMut};
use std::io::{self, Read, Write};
use std::net::TcpStream;

pub struct Handle {
    stream: BufStream<TcpStream>,
    buffer: BytesMut,
}

impl Handle {
    pub fn new(socket: TcpStream) -> Handle {
        Handle {
            stream: BufStream::new(socket),
            buffer: BytesMut::with_capacity(256),
        }
    }

    // Reads a frame from the buffer
    fn parse_frame(&mut self) -> Option<Frame> {
        if self.buffer.is_empty() {
            return None;
        }

        match self.buffer.get_u8() {
            0x1 => {
                let name = String::from_utf8(self.buffer.to_vec()).unwrap();
                Some(Frame::Name(name))
            }
            _ => todo!(),
        }
    }

    // Reads a frame from the TcpStream
    pub fn read_frame(&mut self) -> Result<Option<Frame>, io::Error> {
        if 0 == self.stream.read(&mut self.buffer)? {
            return Ok(None); // Client disconnected
        }

        return Ok(self.parse_frame());
    }

    // Sends a frame on the TcpStream
    pub fn send_frame(&mut self, frame: Frame) -> Result<(), io::Error> {
        match frame {
            Frame::Name(name) => write!(self.stream, "{}{}\n", 0x1, name)?,
            _ => todo!(),
        }

        Ok(())
    }
}
