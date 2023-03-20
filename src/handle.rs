// Used to send messages to the client
// Async TcpStream
use crate::frame::Frame;
use tokio::{net::TcpStream, io::AsyncWriteExt};
use tokio::io::{BufWriter, AsyncReadExt};
use bytes::{BufMut, BytesMut};

pub struct Handle {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Handle {
    pub fn new(socket: TcpStream) -> Handle {
        Handle {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(1024),
        }
    }

    pub async fn read_frame(&mut self) -> Frame {
        match self.net_read().await {
            1 => Frame::Name(String::from_utf8(self.buffer.to_vec()).unwrap()),
            _ => panic!("Received unsupported packet!"),
        }
    }

    async fn net_read(&mut self) -> u8 {
        let mut header: [u8; 2] = [0; 2];

        self.stream.read_exact(&mut header).await.unwrap();

        let len = header[0];
        let flag = header[1];

        let mut data = vec![0u8, len];

        self.stream.read_exact(&mut data).await.unwrap();

        self.buffer.clear();
        self.buffer.put(data.as_slice());

        flag
    }

    pub async fn send_frame(&mut self, frame: Frame) {
        let len = frame_len(&frame);

        match frame {
            Frame::Name(name) => {
                self.stream.write_u8(len).await.unwrap();
                self.stream.write_u8(1).await.unwrap();
                self.stream.write_all(name.as_bytes()).await.unwrap();
            }
            Frame::Start(_) => todo!(),
        }
    }
}

fn frame_len(frame: &Frame) -> u8 {
    match frame {
        Frame::Name(name) => name.len().try_into().unwrap(),
        Frame::Start(_) => todo!(),
    }
}