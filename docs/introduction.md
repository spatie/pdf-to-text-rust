---
title: Introduction
weight: 1
---

This package provides a class to extract text from a pdf.
It is a Rust port of [spatie/pdf-to-text](https://github.com/spatie/pdf-to-text),
using the installed Poppler or Xpdf `pdftotext` program.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use spatie_pdf_to_text::Pdf;
# std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures"))?;

println!("{}", Pdf::get_text("book.pdf")?); // returns the text from the pdf
# assert_eq!(Pdf::get_text("book.pdf")?, "This is a dummy PDF");
# Ok(())
# }
```

Continue with [installation](installation-and-setup.md) and [usage](usage.md).
