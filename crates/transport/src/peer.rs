use either::Either;
use tokio::{
    io::{AsyncRead, AsyncWrite, Stdin, Stdout, stdin, stdout},
    pin,
    process::{Child, ChildStdin, ChildStdout},
};
use triomphe::Arc;

/// A peer represents a communication endpoint, either as a child process or the
/// current process.
///
/// It encapsulates the input and output streams (`incoming` and `outgoing`) and
/// optionally manages a child process (`process`). The streams are wrapped in
/// `Either` to handle both child process streams and standard I/O streams.
pub struct Peer {
    process:  Option<Child>,
    incoming: Either<ChildStdout, Stdin>,
    outgoing: Either<ChildStdin, Stdout>,
}

/// A reader for a peer's incoming data stream.
///
/// This struct holds the input stream (`incoming`) and optionally a reference
/// to the child process (`_process`) to ensure the process is not dropped while
/// the reader is active.
pub struct Reader {
    _process: Option<Arc<Child>>,
    incoming: Either<ChildStdout, Stdin>,
}

/// A writer for a peer's outgoing data stream.
///
/// This struct holds the output stream (`outgoing`) and optionally a reference
/// to the child process (`_process`) to ensure the process is not dropped while
/// the writer is active.
pub struct Writer {
    _process: Option<Arc<Child>>,
    outgoing: Either<ChildStdin, Stdout>,
}

impl Default for Peer {
    /// Creates a default `Peer` instance using the current process's standard
    /// I/O streams.
    fn default() -> Self {
        Self::new()
    }
}

impl Peer {
    /// Creates a new `Peer` instance using the current process's standard I/O
    /// streams.
    ///
    /// This is equivalent to creating a peer that communicates via `stdin` and
    /// `stdout`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            process:  None,
            incoming: Either::Right(stdin()),
            outgoing: Either::Right(stdout()),
        }
    }

    /// Splits the `Peer` into separate `Reader` and `Writer` instances.
    ///
    /// This allows concurrent reading and writing operations. The `Reader` and
    /// `Writer` share ownership of the child process (if any) to ensure it
    /// remains alive while either is in use.
    ///
    /// # Returns
    /// A tuple containing the `Reader` and `Writer` instances.
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

    /// Creates a new `Peer` instance from a child process.
    ///
    /// This method takes ownership of the child process and its standard I/O
    /// streams (`stdin` and `stdout`). If the child process does not have
    /// these streams, the original child process is returned as an error.
    ///
    /// # Errors
    /// Returns the original `Child` if:
    /// - The child process does not have a `stdin` stream.
    /// - The child process does not have a `stdout` stream.
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

    /// Gracefully shuts down the peer by terminating the associated child
    /// process (if any).
    ///
    /// This method ensures the child process is terminated and resources are
    /// cleaned up.
    ///
    /// # Errors
    /// Returns an `std::io::Error` if the child process could not be killed.
    pub async fn close(&mut self) -> Result<(), std::io::Error> {
        if let Some(mut process) = self.process.take() {
            process.kill().await?;
        }
        Ok(())
    }
}

impl TryFrom<Child> for Peer {
    type Error = Child;

    /// Attempts to convert a `Child` process into a `Peer`.
    ///
    /// This is a convenience wrapper around [`Peer::from_child`], allowing
    /// the use of `TryFrom` trait for conversion.
    ///
    /// # Errors
    /// Returns the original `Child` if the conversion fails (see
    /// [`Peer::from_child`]).
    fn try_from(value: Child) -> Result<Self, Self::Error> {
        Self::from_child(value)
    }
}

impl AsyncRead for Reader {
    /// Polls the underlying stream for data to read.
    ///
    /// This delegates to either the child process's `stdout` or the current
    /// process's `stdin`, depending on the configuration of the `Reader`.
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
    /// Polls the underlying stream for readiness to write data.
    ///
    /// This delegates to either the child process's `stdin` or the current
    /// process's `stdout`, depending on the configuration of the `Writer`.
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
    /// Polls the underlying stream for data to read.
    ///
    /// This delegates to either the child process's `stdout` or the current
    /// process's `stdin`, depending on the configuration of the `Peer`.
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
    /// Polls the underlying stream for readiness to write data.
    ///
    /// This delegates to either the child process's `stdin` or the current
    /// process's `stdout`, depending on the configuration of the `Peer`.
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
