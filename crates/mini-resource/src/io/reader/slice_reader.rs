use std::{
    io::SeekFrom,
    pin::Pin,
    task::{Context, Poll},
};

use mini_core::{
    futures_io::{AsyncRead, AsyncSeek},
    futures_lite::ready,
    stackfuture::StackFuture,
};

use super::{Reader, STACK_FUTURE_SIZE};

/// An [`AsyncRead`] implementation capable of reading a [`&[u8]`].
pub struct SliceReader<'a> {
    bytes: &'a [u8],
    bytes_read: usize,
}

impl<'a> SliceReader<'a> {
    /// Create a new [`SliceReader`] for `bytes`.
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bytes_read: 0,
        }
    }
}

impl<'a> AsyncRead for SliceReader<'a> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        if self.bytes_read >= self.bytes.len() {
            Poll::Ready(Ok(0))
        } else {
            let n = ready!(Pin::new(&mut &self.bytes[self.bytes_read..]).poll_read(cx, buf))?;
            self.bytes_read += n;
            Poll::Ready(Ok(n))
        }
    }
}

impl<'a> AsyncSeek for SliceReader<'a> {
    fn poll_seek(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        pos: SeekFrom,
    ) -> Poll<std::io::Result<u64>> {
        let result = match pos {
            SeekFrom::Start(offset) => offset.try_into(),
            SeekFrom::End(offset) => self.bytes.len().try_into().map(|len: i64| len - offset),
            SeekFrom::Current(offset) => self
                .bytes_read
                .try_into()
                .map(|bytes_read: i64| bytes_read + offset),
        };

        if let Ok(new_pos) = result {
            if new_pos < 0 {
                Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "seek position is out of range",
                )))
            } else {
                self.bytes_read = new_pos as _;

                Poll::Ready(Ok(new_pos as _))
            }
        } else {
            Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "seek position is out of range",
            )))
        }
    }
}

impl Reader for SliceReader<'_> {
    fn read_to_end<'a>(
        &'a mut self,
        buf: &'a mut Vec<u8>,
    ) -> StackFuture<'a, std::io::Result<usize>, STACK_FUTURE_SIZE> {
        StackFuture::from(async {
            if self.bytes_read >= self.bytes.len() {
                Ok(0)
            } else {
                buf.extend_from_slice(&self.bytes[self.bytes_read..]);
                let n = self.bytes.len() - self.bytes_read;
                self.bytes_read = self.bytes.len();
                Ok(n)
            }
        })
    }
}
