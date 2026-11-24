# PDF fixtures

The original PDFs come from the MIT-licensed `spatie/pdf-to-text` package at
revision `c732f153c299ef23d9e44dc953c55e31e24e57aa`.
`book.pdf` is a copy of `dummy.pdf`; `table.pdf` is a copy of `scoreboard.pdf`.
These names keep the guide examples close to the PHP README.

`encrypted.pdf` is a derivative of `dummy.pdf`, generated for password tests:

```sh
qpdf --encrypt "user ' \$; secret" 'myP1$$Word' 256 -- dummy.pdf encrypted.pdf
```

These are public test passwords, not credentials.
