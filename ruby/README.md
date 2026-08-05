# anydoc

[![Gem Version](https://img.shields.io/gem/v/anydoc.svg)](https://rubygems.org/gems/anydoc)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/firecrawl/anydoc/blob/main/LICENSE)

Convert Word, PowerPoint, Excel, OpenDocument, RTF, EPUB, CSV, and PDF files into clean GitHub-Flavored Markdown. Ruby bindings for the [anydoc](https://github.com/firecrawl/anydoc) Rust crate, built by [Firecrawl](https://firecrawl.dev). Also available as a hosted API through [Firecrawl Parse](https://firecrawl.dev/parse), which adds OCR for scanned pages anydoc cannot read on its own.

Every format parses into one shared document model and renders through a single Markdown serializer, so headings, tables, lists, and footnotes come out consistently. Conversion runs without holding Ruby's global VM lock, and RBS signatures ship with the gem.

```bash
gem install anydoc
```

Or add it with Bundler:

```bash
bundle add anydoc
```

## Supported formats

| Format           | Extensions                                                 |
| ---------------- | ---------------------------------------------------------- |
| Word             | `.doc`, `.docx`, `.docm`                                   |
| PowerPoint       | `.ppt`, `.pps`, `.pot`, `.pptx`, `.pptm`, `.ppsx`, `.ppsm` |
| Excel            | `.xls`, `.xlsx`, `.xlsm`, `.xlsb`                          |
| OpenDocument     | `.odt`, `.ods`, `.odp`                                     |
| Rich Text Format | `.rtf`                                                     |
| EPUB             | `.epub`                                                    |
| CSV              | `.csv`                                                     |
| PDF              | `.pdf`                                                     |

## Usage

```ruby
require "anydoc"

# From a file path. Pathname and other #to_path objects are accepted.
markdown = Anydoc.to_markdown("report.docx")

# From bytes, with the format detected from the content.
markdown = Anydoc.to_markdown_bytes(data)

# Signature-less formats such as CSV need an explicit format.
markdown = Anydoc.to_markdown_bytes(data, :csv)

# Or stop at the immutable document model, which carries embedded assets.
document = Anydoc.to_document(data)
document.blocks
document.notes
document.assets
```

## Format detection

The format is read from the file content using the marker designated by its specification: the PDF header, RTF open group, OLE stream names, or ZIP package metadata. CSV has no signature, so detection returns `nil`; its extension or an explicit format names it instead.

```ruby
Anydoc.format_from_bytes(data)              # => :docx, or nil
Anydoc.format_from_extension(".pptm")       # => :pptx
Anydoc.format_from_path("report.odt")       # => :odt
```

## Images and embedded objects

Markdown cannot embed bytes. Embedded images render as alt text while their binary strings remain in `document.assets`, tagged with a media type and the package part they came from. Images with external URLs render as ordinary Markdown images.

PDF conversion emits Markdown directly and does not have a document-model form; use `Anydoc.to_markdown` or `Anydoc.to_markdown_bytes` for PDFs.

## Development

```bash
bin/setup
bundle exec rake compile test
```

## License

[MIT](https://github.com/firecrawl/anydoc/blob/main/LICENSE)
