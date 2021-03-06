use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use async_io::Async;
use futures_io::{AsyncRead, AsyncWrite};
use ssh2::{File, FileStat, OpenFlags, OpenType, RenameFlags, Sftp};

use crate::util::poll_once;

pub struct AsyncSftp<S> {
    inner: Sftp,
    async_io: Arc<Async<S>>,
}

impl<S> AsyncSftp<S> {
    pub(crate) fn from_parts(inner: Sftp, async_io: Arc<Async<S>>) -> Self {
        Self { inner, async_io }
    }
}

impl<S> AsyncSftp<S> {
    pub async fn open_mode(
        &self,
        filename: &Path,
        flags: OpenFlags,
        mode: i32,
        open_type: OpenType,
    ) -> io::Result<AsyncFile<S>> {
        let inner = &self.inner;

        let ret = self
            .async_io
            .write_with(|_| {
                inner
                    .open_mode(filename, flags, mode, open_type)
                    .map_err(|err| err.into())
            })
            .await;

        ret.and_then(|file| Ok(AsyncFile::from_parts(file, self.async_io.clone())))
    }

    pub async fn open(&self, filename: &Path) -> io::Result<AsyncFile<S>> {
        let inner = &self.inner;

        let ret = self
            .async_io
            .write_with(|_| inner.open(filename).map_err(|err| err.into()))
            .await;

        ret.and_then(|file| Ok(AsyncFile::from_parts(file, self.async_io.clone())))
    }

    pub async fn create(&self, filename: &Path) -> io::Result<AsyncFile<S>> {
        let inner = &self.inner;

        let ret = self
            .async_io
            .write_with(|_| inner.create(filename).map_err(|err| err.into()))
            .await;

        ret.and_then(|file| Ok(AsyncFile::from_parts(file, self.async_io.clone())))
    }

    pub async fn opendir(&self, dirname: &Path) -> io::Result<AsyncFile<S>> {
        let inner = &self.inner;

        let ret = self
            .async_io
            .write_with(|_| inner.opendir(dirname).map_err(|err| err.into()))
            .await;

        ret.and_then(|file| Ok(AsyncFile::from_parts(file, self.async_io.clone())))
    }

    pub async fn readdir(&self, dirname: &Path) -> io::Result<Vec<(PathBuf, FileStat)>> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.readdir(dirname).map_err(|err| err.into()))
            .await
    }

    pub async fn mkdir(&self, filename: &Path, mode: i32) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.mkdir(filename, mode).map_err(|err| err.into()))
            .await
    }

    pub async fn rmdir(&self, filename: &Path) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.rmdir(filename).map_err(|err| err.into()))
            .await
    }

    pub async fn stat(&self, filename: &Path) -> io::Result<FileStat> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.stat(filename).map_err(|err| err.into()))
            .await
    }

    pub async fn lstat(&self, filename: &Path) -> io::Result<FileStat> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.lstat(filename).map_err(|err| err.into()))
            .await
    }

    pub async fn setstat(&self, filename: &Path, stat: FileStat) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| {
                inner
                    .setstat(filename, stat.clone())
                    .map_err(|err| err.into())
            })
            .await
    }

    pub async fn symlink(&self, path: &Path, target: &Path) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.symlink(path, target).map_err(|err| err.into()))
            .await
    }

    pub async fn readlink(&self, path: &Path) -> io::Result<PathBuf> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.readlink(path).map_err(|err| err.into()))
            .await
    }

    pub async fn realpath(&self, path: &Path) -> io::Result<PathBuf> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.realpath(path).map_err(|err| err.into()))
            .await
    }

    pub async fn rename(
        &self,
        src: &Path,
        dst: &Path,
        flags: Option<RenameFlags>,
    ) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.rename(src, dst, flags).map_err(|err| err.into()))
            .await
    }

    pub async fn unlink(&self, file: &Path) -> io::Result<()> {
        let inner = &self.inner;

        self.async_io
            .write_with(|_| inner.unlink(file).map_err(|err| err.into()))
            .await
    }
}

//
//
//
pub struct AsyncFile<S> {
    inner: File,
    async_io: Arc<Async<S>>,
}

impl<S> AsyncFile<S> {
    pub(crate) fn from_parts(inner: File, async_io: Arc<Async<S>>) -> Self {
        Self { inner, async_io }
    }
}

impl<S> AsyncRead for AsyncFile<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();

        let inner = &mut this.inner;

        poll_once(cx, this.async_io.read_with(|_| inner.read(buf)))
    }
}

impl<S> AsyncWrite for AsyncFile<S> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<io::Result<usize>> {
        let this = self.get_mut();

        let inner = &mut this.inner;

        poll_once(cx, this.async_io.write_with(|_| inner.write(buf)))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        let this = self.get_mut();

        let inner = &mut this.inner;

        poll_once(cx, this.async_io.write_with(|_| inner.flush()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        let this = self.get_mut();

        let _ = &mut this.inner;

        // TODO
        poll_once(cx, this.async_io.write_with(|_| Ok(())))
    }
}
