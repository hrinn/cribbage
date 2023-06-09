// Used to send messages to the client
// Async TcpStream
use crate::frame::Frame;
use crate::game::Card;
use crate::game::Hand;
use bytes::{BufMut, BytesMut};
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;

pub struct Handle {
    reader: BufReader<TcpStream>,
    stream: TcpStream,
}

impl Handle {
    pub fn new(stream: TcpStream) -> Handle {
        Handle {
            reader: BufReader::new(stream.try_clone().unwrap()),
            stream: stream,
        }
    }

    // Reads a frame from the TcpStream
    pub fn read_frame(&mut self) -> Result<Option<Frame>, io::Error> {
        let mut buffer = String::new();

        if 0 == self.reader.read_line(&mut buffer)? {
            return Ok(None); // Client disconnected
        }

        parse_frame(buffer.trim())
    }

    // Sends a frame on the TcpStream
    pub fn send_frame(&mut self, frame: &Frame) -> Result<(), io::Error> {
        let mut buffer = BytesMut::with_capacity(256);

        match frame {
            Frame::Name(name) => {
                buffer.put_u8(0x1);
                buffer.put(name.as_bytes());
            }
            Frame::Start(names) => {
                buffer.put_u8(0x2);

                for name in names {
                    buffer.put(format!("{},", name).as_bytes());
                }
            }
            Frame::Hand(hand) => {
                buffer.put_u8(0x3);

                if let Some(magic) = hand.magic() {
                    buffer.put_u8(0x1);
                    buffer.put(magic.to_net_name().as_bytes());
                } else {
                    buffer.put_u8(0x0);
                    buffer.put_slice(b"00");
                }

                for card in hand.cards() {
                    buffer.put(format!("{},", card.to_net_name()).as_bytes());
                }
            }
            Frame::Card(card) => {
                buffer.put_u8(0x4);
                buffer.put(card.to_net_name().as_bytes());
            }
            Frame::Play(card, out) => {
                buffer.put_u8(0x5);
                buffer.put_u8(if *out { 0x1 } else { 0x0 });
                if let Some(card) = card {
                    buffer.put(card.to_net_name().as_bytes());
                } else {
                    buffer.put_slice(b"go");
                }
            }
            Frame::RoundDone => {
                buffer.put_u8(0x6);
            }
            Frame::Seed(seed) => {
                buffer.put_u8(0x7);
                buffer.put(seed.as_bytes());
            }
        }

        buffer.put_slice(b"\n");
        self.stream.write_all(&buffer)?;
        self.stream.flush()?;

        Ok(())
    }
}

fn parse_frame(buffer: &str) -> Result<Option<Frame>, io::Error> {
    if buffer.is_empty() {
        return Ok(None);
    }

    match buffer.as_bytes().get(0).unwrap() {
        0x1 => Ok(Some(Frame::Name(buffer[1..].to_string()))),
        0x2 => Ok(Some(Frame::Start(
            buffer[1..]
                .strip_suffix(',')
                .unwrap()
                .split(',')
                .map(|str| String::from(str))
                .collect(),
        ))),
        0x3 => {
            let magic = if buffer[1..2].as_bytes().get(0).unwrap() == &0x1 {
                Some(Card::from_net_name(buffer[2..4].to_string()))
            } else {
                None
            };

            Ok(Some(Frame::Hand(Hand::from(
                buffer[4..]
                    .strip_suffix(',')
                    .unwrap()
                    .split(',')
                    .map(|str| Card::from_net_name(str.to_string()))
                    .collect(),
                magic,
            ))))
        }
        0x4 => Ok(Some(Frame::Card(Card::from_net_name(
            buffer[1..].to_string(),
        )))),
        0x5 => {
            let card = if buffer[2..].eq("go") {
                None
            } else {
                Some(Card::from_net_name(buffer[2..].to_string()))
            };

            let out = buffer[1..2].as_bytes().get(0).unwrap() == &0x1;

            Ok(Some(Frame::Play(card, out)))
        }
        0x6 => Ok(Some(Frame::RoundDone)),
        0x7 => Ok(Some(Frame::Seed(buffer[1..].to_string()))),
        _ => Err(io::ErrorKind::InvalidData.into()),
    }
}
