// Used to send messages to the client
// Async TcpStream
use crate::frame::Frame;
use bytes::{Buf, BufMut, BytesMut};
use std::borrow::BorrowMut;
use std::io::{self, Read, Write};
use std::net::TcpStream;

pub struct Handle {
    stream: TcpStream,
    buffer: BytesMut,
}

impl Handle {
    pub fn new(stream: TcpStream) -> Handle {
        Handle {
            stream: stream,
            buffer: BytesMut::with_capacity(256),
        }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>, io::Error> {
        println!("Parsing frame");

        if self.buffer.is_empty() {
            return Ok(None);
        }

        match self.buffer.get_u8() {
            0x1 => Ok(Some(Frame::Name(
                String::from_utf8(self.buffer.to_vec().clone()).unwrap(),
            ))),
            0x2 => Ok(Some(Frame::Start(
                String::from_utf8(self.buffer.to_vec())
                    .unwrap()
                    .split(',')
                    .map(|str| String::from(str))
                    .collect(),
            ))),
            _ => Err(io::ErrorKind::InvalidData.into()),
        }
    }

    // Reads a frame from the TcpStream
    pub fn read_frame(&mut self) -> Result<Option<Frame>, io::Error> {
        self.buffer.clear();

        println!("read");

        if 0 == self.stream.read_to_end(self.buffer)? {
            return Ok(None); // Client disconnected
        }

        println!("Non zero read");

        self.parse_frame()
    }

    // Sends a frame on the TcpStream
    pub fn send_frame(&mut self, frame: &Frame) -> Result<(), io::Error> {
        match frame {
            Frame::Name(name) => {
                self.buffer.put_u8(0x1);
                self.buffer.put(name.as_bytes());
                self.buffer.put_slice(b"\n");
            }
            Frame::Start(names) => {
                self.buffer.put_u8(0x2);

                for name in names {
                    self.buffer.put(format!("{name},").as_bytes());
                }
            }
        }

        self.stream.write_all(&self.buffer)?;
        self.stream.flush()?;
        self.buffer.clear();

        Ok(())
    }
}
