use either::Either;
use tokio::{
    io::{AsyncRead, AsyncWrite, Stdin, Stdout, stdin, stdout},
    pin,
    process::{Child, ChildStdin, ChildStdout},
};
use triomphe::Arc;

pub struct Peer {
    process:  Option<Child>,
    incoming: Either<ChildStdout, Stdin>,
    outgoing: Either<ChildStdin, Stdout>,
}

pub struct Reader {
    _process: Option<Arc<Child>>,
    incoming: Either<ChildStdout, Stdin>,
}

pub struct Writer {
    _process: Option<Arc<Child>>,
    outgoing: Either<ChildStdin, Stdout>,
}

impl Default for Peer {
    fn default() -> Self {
        Self::new()
    }
}

impl Peer {
    /// Create a new peer from this process.
    #[must_use]
    pub fn new() -> Self {
        Self {
            process:  None,
            incoming: Either::Right(stdin()),
            outgoing: Either::Right(stdout()),
        }
    }

    pub fn split(self) -> (Reader, Writer) {
        let Self {
            incoming,
            outgoing,
            process,
        } = self;
        let process = process.map(Arc::new);
        (
            Reader {
                _process: process.clone(),
                incoming,
            },
            Writer {
                _process: process,
                outgoing,
            },
        )
    }

    /// Create a new peer from a child process.
    ///
    /// # Errors
    /// Returns child it self if the child process does not have a stdin or
    /// stdout.
    #[allow(clippy::result_large_err)]
    pub fn from_child(mut child: Child) -> Result<Self, Child> {
        let Some(stdin) = child.stdin.take() else {
            return Err(child);
        };
        let Some(stdout) = child.stdout.take() else {
            return Err(child);
        };

        Ok(Self {
            process:  Some(child),
            incoming: Either::Left(stdout),
            outgoing: Either::Left(stdin),
        })
    }

    /// Close the peer.
    ///
    /// # Errors
    /// Returns an error if the child process could not be killed.
    pub async fn close(&mut self) -> Result<(), std::io::Error> {
        if let Some(mut process) = self.process.take() {
            process.kill().await?;
        }
        Ok(())
    }
}

impl TryFrom<Child> for Peer {
    type Error = Child;

    fn try_from(value: Child) -> Result<Self, Self::Error> {
        Self::from_child(value)
    }
}

impl AsyncRead for Reader {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut().incoming {
            Either::Left(ref mut stdout) => {
                pin!(stdout);
                stdout.poll_read(cx, buf)
            }
            Either::Right(ref mut stdin) => {
                pin!(stdin);
                stdin.poll_read(cx, buf)
            }
        }
    }
}

impl AsyncWrite for Writer {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match self.get_mut().outgoing {
            Either::Left(ref mut stdin) => {
                pin!(stdin);
                stdin.poll_write(cx, buf)
            }
            Either::Right(ref mut stdout) => {
                pin!(stdout);
                stdout.poll_write(cx, buf)
            }
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut().outgoing {
            Either::Left(ref mut stdin) => {
                pin!(stdin);
                stdin.poll_flush(cx)
            }
            Either::Right(ref mut stdout) => {
                pin!(stdout);
                stdout.poll_flush(cx)
            }
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut().outgoing {
            Either::Left(ref mut stdin) => {
                pin!(stdin);
                stdin.poll_shutdown(cx)
            }
            Either::Right(ref mut stdout) => {
                pin!(stdout);
                stdout.poll_shutdown(cx)
            }
        }
    }
}

impl AsyncRead for Peer {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut().incoming {
            Either::Left(ref mut stdout) => {
                pin!(stdout);
                stdout.poll_read(cx, buf)
            }
            Either::Right(ref mut stdin) => {
                pin!(stdin);
                stdin.poll_read(cx, buf)
            }
        }
    }
}

impl AsyncWrite for Peer {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match self.get_mut().outgoing {
            Either::Left(ref mut stdin) => {
                pin!(stdin);
                stdin.poll_write(cx, buf)
            }
            Either::Right(ref mut stdout) => {
                pin!(stdout);
                stdout.poll_write(cx, buf)
            }
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut().outgoing {
            Either::Left(ref mut stdin) => {
                pin!(stdin);
                stdin.poll_flush(cx)
            }
            Either::Right(ref mut stdout) => {
                pin!(stdout);
                stdout.poll_flush(cx)
            }
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut().outgoing {
            Either::Left(ref mut stdin) => {
                pin!(stdin);
                stdin.poll_shutdown(cx)
            }
            Either::Right(ref mut stdout) => {
                pin!(stdout);
                stdout.poll_shutdown(cx)
            }
        }
    }
}
