#![doc = include_str!("../README.md")]

use std::{
    ffi::OsString,
    fmt, fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    time::Duration,
};

use wait_timeout::ChildExt;

/// A failure to locate, read, run, or decode a PDF conversion.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// No executable was found in the common locations or PATH.
    #[error("The required pdftotext binary was not found or is not executable")]
    BinaryNotFound,
    /// The PDF could not be opened for reading.
    #[error("Could not read `{path}`")]
    PdfNotFound {
        /// The requested PDF.
        path: PathBuf,
        /// The underlying filesystem error.
        #[source]
        source: io::Error,
    },
    /// No PDF has been configured.
    #[error("No PDF has been configured; call set_pdf first")]
    PdfNotSet,
    /// Starting, waiting for, or reading the process failed.
    #[error("Could not run pdftotext: {0}")]
    Process(#[from] io::Error),
    /// The process exceeded its configured timeout and was killed and reaped.
    #[error("pdftotext exceeded its timeout of {0:?}")]
    Timeout(Duration),
    /// The converter exited unsuccessfully.
    #[error(
        "Could not extract text ({status}): {}",
        String::from_utf8_lossy(stderr)
    )]
    CouldNotExtractText {
        /// The process exit status.
        status: ExitStatus,
        /// Standard error, without a command line or password arguments.
        stderr: Vec<u8>,
    },
    /// The selected output encoding is not UTF-8; use `text_bytes` instead.
    #[error("pdftotext output is not UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}

/// Extract text using an installed Poppler or Xpdf `pdftotext` executable.
///
/// Configuration methods consume and return the value. Extraction borrows it,
/// so a configured converter can be reused. Commands never pass through a shell.
#[must_use]
pub struct Pdf {
    binary: PathBuf,
    pdf: Option<PathBuf>,
    options: Vec<OsString>,
    timeout: Duration,
}

impl fmt::Debug for Pdf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pdf")
            .field("binary", &self.binary)
            .field("pdf", &self.pdf)
            .field("options", &"[redacted]")
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl Pdf {
    /// Discover `pdftotext` in the PHP package's common locations, then PATH.
    ///
    /// # Errors
    /// Returns [`Error::BinaryNotFound`] when no executable is available.
    pub fn new() -> Result<Self, Error> {
        let common = [
            "/usr/bin/pdftotext",
            "/usr/local/bin/pdftotext",
            "/opt/homebrew/bin/pdftotext",
            "/opt/local/bin/pdftotext",
            r"C:\Program Files\xpdf-tools-win\bin64\pdftotext.exe",
        ];
        let paths = common.into_iter().map(PathBuf::from).chain(
            std::env::var_os("PATH")
                .into_iter()
                .flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
                .map(|path| {
                    path.join(if cfg!(windows) {
                        "pdftotext.exe"
                    } else {
                        "pdftotext"
                    })
                }),
        );
        for path in paths {
            if executable(&path) {
                return Ok(Self::with_binary(path));
            }
        }
        Err(Error::BinaryNotFound)
    }

    /// Use a binary path or a command name resolved through PATH at execution.
    /// Existence and executability are checked when extracting text.
    pub fn with_binary(binary: impl Into<PathBuf>) -> Self {
        Self {
            binary: binary.into(),
            pdf: None,
            options: Vec::new(),
            timeout: Duration::from_secs(60),
        }
    }

    /// Set a readable input PDF.
    ///
    /// # Errors
    /// Returns [`Error::PdfNotFound`] if the path cannot be opened as a file.
    pub fn set_pdf(mut self, pdf: impl Into<PathBuf>) -> Result<Self, Error> {
        let pdf = pdf.into();
        fs::File::open(&pdf)
            .and_then(|file| {
                if file.metadata()?.is_file() {
                    Ok(())
                } else {
                    Err(io::Error::new(io::ErrorKind::InvalidInput, "not a file"))
                }
            })
            .map_err(|source| Error::PdfNotFound {
                path: pdf.clone(),
                source,
            })?;
        self.pdf = Some(pdf);
        Ok(self)
    }

    /// Replace all options. A leading hyphen is optional. Each string splits
    /// at its first ASCII space only, preserving spaces and quotes in values.
    pub fn set_options(mut self, options: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        self.options.clear();
        self.add_options(options)
    }

    /// Append options without replacing earlier options.
    pub fn add_options(mut self, options: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        for option in options {
            let option = option.as_ref().trim_matches(php_option_whitespace);
            let (flag, value) = option
                .split_once(' ')
                .map_or((option, None), |(flag, value)| (flag, Some(value)));
            self.options.push(if flag.starts_with('-') {
                flag.into()
            } else {
                format!("-{flag}").into()
            });
            if let Some(value) = value {
                self.options.push(value.into());
            }
        }
        self
    }

    /// Set the execution timeout (default 60 seconds). Zero disables it.
    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Extract UTF-8 text, trimming the same edge bytes as the PHP package.
    ///
    /// # Errors
    /// Returns an [`Error`] for missing input, process failure, timeout or invalid UTF-8.
    pub fn text(&self) -> Result<String, Error> {
        self.text_with(|_| {})
    }

    /// Customize the native command before execution, for example its environment
    /// or current directory. Output is always captured and stdin is closed.
    ///
    /// # Errors
    /// Returns the same errors as [`Self::text`].
    pub fn text_with(&self, callback: impl FnOnce(&mut Command)) -> Result<String, Error> {
        Ok(String::from_utf8(self.text_bytes_with(callback)?)?)
    }

    /// Extract bytes without requiring UTF-8, for options such as `enc Latin1`.
    /// The same ASCII edge bytes are trimmed as for [`Self::text`].
    ///
    /// # Errors
    /// Returns an [`Error`] for missing input, process failure or timeout.
    pub fn text_bytes(&self) -> Result<Vec<u8>, Error> {
        self.text_bytes_with(|_| {})
    }

    /// Extract bytes after customizing the native command.
    ///
    /// # Errors
    /// Returns the same errors as [`Self::text_bytes`].
    pub fn text_bytes_with(&self, callback: impl FnOnce(&mut Command)) -> Result<Vec<u8>, Error> {
        let pdf = self.pdf.as_ref().ok_or(Error::PdfNotSet)?;
        // Resolve before a callback changes the child's working directory. This
        // also prevents a relative filename beginning with '-' becoming a flag.
        let pdf = fs::canonicalize(pdf).map_err(|source| Error::PdfNotFound {
            path: pdf.clone(),
            source,
        })?;
        let mut command = Command::new(&self.binary);
        command.args(&self.options).arg(pdf).arg("-");
        callback(&mut command);
        // Files avoid pipe-buffer deadlocks and reader threads that could outlive
        // a timed-out child when descendants inherit its output handles.
        let mut stdout = tempfile::tempfile()?;
        let mut stderr = tempfile::tempfile()?;
        command
            .stdin(Stdio::null())
            .stdout(stdout.try_clone()?)
            .stderr(stderr.try_clone()?);
        let mut child = command.spawn()?;
        let status = if self.timeout.is_zero() {
            child.wait()?
        } else {
            match child.wait_timeout(self.timeout) {
                Ok(Some(status)) => status,
                result => {
                    let _ = child.kill();
                    child.wait()?;
                    result?;
                    return Err(Error::Timeout(self.timeout));
                }
            }
        };
        use io::{Read, Seek};
        if !status.success() {
            stderr.rewind()?;
            let mut bytes = Vec::new();
            stderr.read_to_end(&mut bytes)?;
            return Err(Error::CouldNotExtractText {
                status,
                stderr: bytes,
            });
        }
        stdout.rewind()?;
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes)?;
        let edge = |byte: &u8| matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0 | 11 | 12);
        let start = bytes
            .iter()
            .position(|byte| !edge(byte))
            .unwrap_or(bytes.len());
        let end = bytes
            .iter()
            .rposition(|byte| !edge(byte))
            .map_or(start, |index| index + 1);
        Ok(bytes[start..end].to_vec())
    }

    /// Extract text with automatic binary discovery and default settings.
    /// Use a configured [`Pdf`] for a custom binary, options, timeout or callback.
    ///
    /// # Errors
    /// Returns the same errors as [`Self::new`], [`Self::set_pdf`] and [`Self::text`].
    pub fn get_text(pdf: impl Into<PathBuf>) -> Result<String, Error> {
        Self::new()?.set_pdf(pdf)?.text()
    }
}

fn php_option_whitespace(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\n' | '\r' | '\0' | '\u{b}')
}

fn executable(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        metadata.is_file()
    }
}

#[cfg(doctest)]
mod guide {
    include!(concat!(env!("OUT_DIR"), "/documentation.rs"));
}
