//! Public extraction behavior against the real installed converter.

use std::{path::PathBuf, process::Command, time::Duration};

use spatie_pdf_to_text::{Error, Pdf};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn extracts_text_and_handles_platform_paths() {
    for name in [
        "dummy.pdf",
        "dummy with spaces in its name.pdf",
        "dummy's_file.pdf",
    ] {
        assert_eq!(Pdf::get_text(fixture(name)).unwrap(), "This is a dummy PDF");
    }
}

#[test]
fn preserves_layout_with_prefixed_unprefixed_and_mixed_options() {
    for options in [
        ["layout", "f 1"],
        ["-layout", "-f 1"],
        ["-layout", "f 1"],
        [" \t-layout\r\n", "\0f 1\u{b}"],
    ] {
        let text = Pdf::new()
            .unwrap()
            .set_pdf(fixture("scoreboard.pdf"))
            .unwrap()
            .set_options(options)
            .text()
            .unwrap();
        assert!(text.contains("Charleroi 50      28     13 11 4"), "{text}");
    }
}

#[test]
fn replaces_and_appends_options_and_can_be_reused() {
    let pdf = Pdf::new()
        .unwrap()
        .set_pdf(fixture("multi_page.pdf"))
        .unwrap()
        .set_options(["invalid"])
        .set_options(["layout", "f 2"])
        .add_options(["l 2"]);
    let text = pdf.text().unwrap();
    assert!(
        text.contains("This is page 2")
            && !text.contains("This is page 1")
            && !text.contains("This is page 3"),
        "{text}"
    );
    assert_eq!(text, pdf.text().unwrap());
}

#[test]
fn reports_missing_input_and_binary_and_process_failure() {
    assert!(matches!(Pdf::new().unwrap().text(), Err(Error::PdfNotSet)));
    assert!(matches!(
        Pdf::new().unwrap().set_pdf(fixture("missing.pdf")),
        Err(Error::PdfNotFound { .. })
    ));
    assert!(matches!(
        Pdf::new().unwrap().set_pdf(fixture(".")),
        Err(Error::PdfNotFound { .. })
    ));
    assert!(matches!(
        Pdf::with_binary(fixture("missing-binary"))
            .set_pdf(fixture("dummy.pdf"))
            .unwrap()
            .text(),
        Err(Error::Process(_))
    ));
    let error = Pdf::new()
        .unwrap()
        .set_pdf(fixture("dummy.pdf"))
        .unwrap()
        .set_options(["foo"])
        .text()
        .unwrap_err();
    assert!(
        matches!(error, Error::CouldNotExtractText { status, stderr } if !status.success() && !stderr.is_empty())
    );
}

#[test]
fn customizes_the_process_and_disables_timeout() {
    let text = Pdf::with_binary("pdftotext")
        .set_pdf(fixture("dummy.pdf"))
        .unwrap()
        .set_timeout(Duration::ZERO)
        .text_with(|command| {
            command
                .current_dir(std::env::temp_dir())
                .env("PDF_TO_TEXT_TEST", "yes");
        })
        .unwrap();
    assert_eq!(text, "This is a dummy PDF");
}

#[test]
fn reports_invalid_pdf_with_exit_status_and_stderr() {
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), b"not a PDF").unwrap();
    let error = Pdf::get_text(file.path()).unwrap_err();
    assert!(
        matches!(error, Error::CouldNotExtractText { status, stderr } if status.code() == Some(1) && !stderr.is_empty())
    );
}

#[test]
fn passwords_with_spaces_quotes_and_shell_metacharacters_are_one_argument() {
    let pdf = Pdf::new()
        .unwrap()
        .set_pdf(fixture("encrypted.pdf"))
        .unwrap();
    assert!(matches!(pdf.text(), Err(Error::CouldNotExtractText { .. })));
    assert_eq!(
        pdf.set_options(["upw user ' $; secret"]).text().unwrap(),
        "This is a dummy PDF"
    );
    assert_eq!(
        Pdf::new()
            .unwrap()
            .set_pdf(fixture("encrypted.pdf"))
            .unwrap()
            .set_options(["opw myP1$$Word"])
            .text()
            .unwrap(),
        "This is a dummy PDF"
    );
}

#[test]
fn debug_output_does_not_reveal_passwords() {
    let pdf = Pdf::with_binary("pdftotext").set_options(["opw secret"]);
    assert!(!format!("{pdf:?}").contains("secret"));
}

#[test]
fn leading_hyphens_and_unicode_in_input_paths_are_not_options() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("-résumé ' $(not-a-command).pdf");
    std::fs::copy(fixture("dummy.pdf"), &path).unwrap();
    assert_eq!(Pdf::get_text(path).unwrap(), "This is a dummy PDF");
}

// Python is installed in CI to make process edge cases identical on all hosts.
fn python() -> PathBuf {
    std::env::var_os("PYTHON").map_or_else(|| PathBuf::from("python3"), PathBuf::from)
}

#[test]
fn timeout_kills_and_reaps_the_child() {
    let directory = tempfile::tempdir().unwrap();
    let marker = directory.path().join("should-not-exist");
    let start = std::time::Instant::now();
    let error = Pdf::with_binary(python())
        .set_pdf(fixture("dummy.pdf"))
        .unwrap()
        .set_timeout(Duration::from_millis(150))
        .text_with(|command| {
            *command = Command::new(python());
            command
                .args([
                    "-c",
                    "import time, pathlib, sys; time.sleep(1); pathlib.Path(sys.argv[1]).touch()",
                ])
                .arg(&marker);
        })
        .unwrap_err();
    assert!(matches!(error, Error::Timeout(_)), "{error}");
    assert!(start.elapsed() < Duration::from_secs(3));
    std::thread::sleep(Duration::from_millis(1100));
    assert!(!marker.exists());
}

#[test]
fn trims_only_php_edge_bytes_and_handles_large_output_without_deadlock() {
    let bytes = Pdf::with_binary(python()).set_pdf(fixture("dummy.pdf")).unwrap().text_bytes_with(|command| {
        *command = Command::new(python());
        command.args(["-c", "import sys; sys.stderr.write('E'*200000); sys.stdout.buffer.write(b' \\t\\n\\r\\0\\x0b\\x0c'+b'\\xc2\\xa0'+b'A'*200000+b'\\x1c'+b'\\x0c\\0\\n ')"]);
    }).unwrap();
    assert_eq!(&bytes[..2], b"\xc2\xa0");
    assert_eq!(bytes.len(), 200003);
    assert_eq!(bytes.last(), Some(&0x1c));
}

#[test]
fn exposes_bytes_for_non_utf8_output() {
    let pdf = Pdf::with_binary(python())
        .set_pdf(fixture("dummy.pdf"))
        .unwrap();
    let customize = |command: &mut Command| {
        *command = Command::new(python());
        command.args(["-c", "import sys; sys.stdout.buffer.write(b'caf\\xe9\\n')"]);
    };
    assert_eq!(pdf.text_bytes_with(customize).unwrap(), b"caf\xe9");
    assert!(matches!(
        pdf.text_with(customize),
        Err(Error::InvalidUtf8(_))
    ));
}
