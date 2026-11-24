---
title: Usage
weight: 4
---

Extracting text from a pdf is easy.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# use spatie_pdf_to_text::Pdf;
# std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"))?;
let text = Pdf::new()?
    .set_pdf("book.pdf")?
    .text()?;
# assert_eq!(text, "This is a dummy PDF");
# Ok(())
# }
```

Or easier:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# use spatie_pdf_to_text::Pdf;
# std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"))?;
println!("{}", Pdf::get_text("book.pdf")?);
# assert_eq!(Pdf::get_text("book.pdf")?, "This is a dummy PDF");
# Ok(())
# }
```

By default the package searches common Linux, Homebrew,
MacPorts and Windows Xpdf locations, then PATH. If the binary is located elsewhere,
pass its path to `with_binary`:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# use spatie_pdf_to_text::Pdf;
# std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"))?;
# let output = std::process::Command::new(if cfg!(windows) { "where.exe" } else { "which" }).arg("pdftotext").output()?;
# let installed = String::from_utf8(output.stdout)?;
# let binary = installed.lines().next().ok_or("pdftotext not found")?;
let text = Pdf::with_binary(binary)
    .set_pdf("book.pdf")?
    .text()?;
# assert_eq!(text, "This is a dummy PDF");
# Ok(())
# }
```

`binary` can be a full path such as `/custom/path/to/pdftotext` or a command name
such as `pdftotext`. On Windows it can be a path containing spaces, such as
`C:\Program Files\xpdf-tools-win\bin64\pdftotext.exe`. Do not add shell quotes.

Sometimes you may want to use [pdftotext options](https://linux.die.net/man/1/pdftotext).
To do so you can set them up using the `set_options` method.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# use spatie_pdf_to_text::Pdf;
# std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"))?;
let text = Pdf::new()?
    .set_pdf("table.pdf")?
    .set_options(["layout", "r 96"])
    .text()?;
# assert!(text.contains("Charleroi 50      28     13 11 4"));
# Ok(())
# }
```

A leading hyphen is optional. Each option string is trimmed and split at its
first ASCII space only. `"opw myP1$$Word"` becomes two arguments, `"-opw"` and
`"myP1$$Word"`. Spaces, quotes and shell metacharacters in the value remain literal.
There is no shell expansion, piping or redirection.

For a password-protected PDF:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# use spatie_pdf_to_text::Pdf;
# std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"))?;
let text = Pdf::new()?
    .set_pdf("encrypted.pdf")?
    .set_options(["opw myP1$$Word"])
    .text()?;
# assert_eq!(text, "This is a dummy PDF");
# Ok(())
# }
```

Use `upw` for a user password. Passwords are native process arguments and may be
visible to other users through operating-system process inspection. `Pdf`'s
debug output redacts options; errors do not include the command line. A custom
binary's own stderr can still contain sensitive information.

Please note that successive calls to `set_options` will overwrite options passed
in during previous calls.

If you need to make multiple calls to add options (for example if you need to
pass in default options when creating the `Pdf` object, and then add
context-specific options elsewhere), you can use the `add_options` method:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# use spatie_pdf_to_text::Pdf;
# std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"))?;
let text = Pdf::new()?
    .set_pdf("table.pdf")?
    .set_options(["layout", "r 96"])
    .add_options(["f 1"])
    .text()?;
# assert!(text.contains("Charleroi 50      28     13 11 4"));
# Ok(())
# }
```

## Timeouts and process customization

The default timeout is 60 seconds. Use `Duration::ZERO` to disable it.
The process callback can set environment variables, change the working directory,
or otherwise customize a `std::process::Command` before it starts.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# use spatie_pdf_to_text::Pdf;
# std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"))?;
use std::time::Duration;

let text = Pdf::new()?
    .set_pdf("book.pdf")?
    .set_timeout(Duration::from_secs(120))
    .text_with(|command| {
        command.env("LC_ALL", "C");
    })?;
# assert_eq!(text, "This is a dummy PDF");
# Ok(())
# }
```

The timeout kills and reaps the direct child, not any descendants a custom binary
might start. Standard output and error are captured in anonymous temporary files
to avoid pipe-buffer deadlocks. Stdin is closed, even when a callback configures
other streams. Do not pass untrusted converter options; options are forwarded
to the installed program without restricting its capabilities.

## Output and errors

`text` returns a UTF-8 `String`. It trims only spaces, tabs, LF, CR,
NUL, vertical tabs and form feeds from the beginning and end. It does not trim
Unicode whitespace or change interior page breaks or line endings. Extraction
behavior and supported options depend on the installed Poppler or Xpdf version.

For another output encoding, use `text_bytes` (or `text_bytes_with` with a process
callback) to receive the same trimmed output as `Vec<u8>` without UTF-8 decoding.

Failures return typed `Error` variants for a missing binary, unreadable or unset
PDF, process I/O, timeout, unsuccessful exit (including status and stderr), and
invalid UTF-8. File readability is checked by `set_pdf`; binary execution and
options are checked by the converter when extracting text.

Configuration methods consume and return `Self`; keep their returned value.
Extraction borrows the configured converter so it can be reused. Input paths
become absolute before execution, preventing a filename from being read as an
option. Directories are rejected.
