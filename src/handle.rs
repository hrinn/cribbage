// Used to send messages to the client
// Async TcpStream
use crate::frame::Frame;
use std::io::{self, BufRead, BufReader, LineWriter, Write};
use std::net::TcpStream;

pub struct Handle {
    reader: BufReader<TcpStream>,
    writer: LineWriter<TcpStream>,
}

impl Handle {
    pub fn new(socket: TcpStream) -> Handle {
        Handle {
            reader: BufReader::new(socket.try_clone().unwrap()),
            writer: LineWriter::new(socket),
        }
    }

    // Reads a frame from the TcpStream
    pub fn read_frame(&mut self) -> Result<Option<Frame>, io::Error> {
        let mut buf = String::new();

        let n = self.reader.read_line(&mut buf)?;

        if n == 0 {
            return Ok(None); // Client disconnected
        }

        return Ok(parse_frame(buf));
    }

    // Sends a frame on the TcpStream
    pub fn send_frame(&mut self, frame: Frame) -> Result<(), io::Error> {
        match frame {
            Frame::Name(name) => {
                self.writer.write(&[0x1])?;
                self.writer.write_all(name.as_bytes())?;
                self.writer.write_all(b"\n")?;
            }
            _ => todo!(),
        }

        Ok(())
    }
}

fn parse_frame(buf: String) -> Option<Frame> {
    if buf.is_empty() {
        return None;
    }

    match buf.as_bytes().get(0).unwrap() {
        0x1 => Some(Frame::Name(buf[1..].trim().to_string())),
        _ => todo!(),
    }
}
