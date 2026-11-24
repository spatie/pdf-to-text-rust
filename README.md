# Extract text from a pdf

[![Latest version on crates.io](https://img.shields.io/crates/v/spatie-pdf-to-text.svg?style=flat-square)](https://crates.io/crates/spatie-pdf-to-text)
[![Tests](https://img.shields.io/github/actions/workflow/status/spatie/pdf-to-text-rust/ci.yml?branch=main&style=flat-square)](https://github.com/spatie/pdf-to-text-rust/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](https://github.com/spatie/pdf-to-text-rust/blob/main/LICENSE.md)
[![Total downloads](https://img.shields.io/crates/d/spatie-pdf-to-text.svg?style=flat-square)](https://crates.io/crates/spatie-pdf-to-text)

This package provides a class to extract text from a pdf, using the installed
Poppler or Xpdf `pdftotext` program. It is a Rust port of
[spatie/pdf-to-text](https://github.com/spatie/pdf-to-text).

## Support us

[<img src="https://spatie.be/github-ad-image/pdf-to-text-rust.jpg" alt="Support Spatie's open source work" width="419px" />](https://spatie.be/github-ad-click/pdf-to-text-rust)

We invest a lot of resources into creating [best in class open source packages](https://spatie.be/open-source). You can support us by [buying one of our paid products](https://spatie.be/open-source/support-us).

We highly appreciate you sending us a postcard from your hometown, mentioning which of our package(s) you are using. You'll find our address on [our contact page](https://spatie.be/about-us). We publish all received postcards on [our virtual postcard wall](https://spatie.be/open-source/postcards).

## Installation

Requires Rust 1.89 or newer and `pdftotext`.

On Ubuntu or Debian, install the binary using `apt-get install poppler-utils`.
On macOS, use `brew install poppler`. On Windows, download the
[Xpdf command line tools](https://www.xpdfreader.com/download.html) and extract
them to `C:\Program Files\xpdf-tools-win`, or add the directory containing
`pdftotext.exe` to PATH.

```sh
cargo add spatie-pdf-to-text
```

## Usage

Extracting text from a pdf is easy.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use spatie_pdf_to_text::Pdf;
# std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"))?;

let text = Pdf::get_text("book.pdf")?;
# assert_eq!(text, "This is a dummy PDF");
# Ok(())
# }
```

See the [documentation](https://spatie.be/docs/pdf-to-text-rust/v1/introduction)
for custom binary paths, options, passwords, timeouts and process customization.

## Testing

```sh
cargo test --locked --all-targets --all-features
cargo test --locked --doc --all-features
```

See [all quality checks](https://github.com/spatie/pdf-to-text-rust/blob/main/docs/testing.md). The README and Markdown guides are tested by rustdoc, including setup on hidden `# ` lines.

## Changelog

See [CHANGELOG.md](https://github.com/spatie/pdf-to-text-rust/blob/main/CHANGELOG.md).

## Contributing

See [CONTRIBUTING.md](https://github.com/spatie/pdf-to-text-rust/blob/main/CONTRIBUTING.md) and the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

Using an AI coding agent? Load Apollo's [rust-best-practices skill](https://github.com/apollographql/skills/tree/main/skills/rust-best-practices) when writing or reviewing code in this package.

## Security Vulnerabilities

Report vulnerabilities privately using the instructions in [SECURITY.md](https://github.com/spatie/pdf-to-text-rust/blob/main/SECURITY.md).

## Credits

- [Freek Van der Herten](https://github.com/freekmurze)
- [All contributors](https://github.com/spatie/pdf-to-text-rust/graphs/contributors)

## License

The MIT License (MIT). See [LICENSE.md](https://github.com/spatie/pdf-to-text-rust/blob/main/LICENSE.md).
