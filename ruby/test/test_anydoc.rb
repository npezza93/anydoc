# frozen_string_literal: true

require "test_helper"

class TestAnydoc < Minitest::Test
  FIXTURES = File.expand_path("../../tests/fixtures", __dir__)
  OUTLINE = File.join(FIXTURES, "docx/handmade-outline.docx")
  RICH = File.join(FIXTURES, "docx/handmade-rich.docx")
  CSV = File.join(FIXTURES, "csv/sheet.csv")

  def test_has_a_version_number
    refute_nil Anydoc::VERSION
  end

  def test_to_markdown_detects_the_format_from_file_content
    assert_match(/^# /, Anydoc.to_markdown(Pathname(OUTLINE)))
  end

  def test_to_markdown_bytes_converts_in_memory
    markdown = Anydoc.to_markdown_bytes(File.binread(RICH), :docx)
    assert_includes markdown, "| Quarter | Widgets |"
  end

  def test_to_markdown_bytes_detects_format_when_none_is_named
    assert_includes Anydoc.to_markdown_bytes(File.binread(RICH)), "| Quarter | Widgets |"

    error = assert_raises(Anydoc::ConvertError) do
      Anydoc.to_markdown_bytes(File.binread(CSV))
    end
    assert_match(/unrecognized file content/, error.message)
    assert_includes Anydoc.to_markdown_bytes(File.binread(CSV), :csv), "| --- |"
  end

  def test_to_document_exposes_the_complete_document_model
    document = Anydoc.to_document(File.binread(OUTLINE), :docx)
    heading = document.blocks.find { _1.kind == "heading" }

    assert_instance_of Anydoc::Document, document
    assert_instance_of Anydoc::Block, heading
    assert_includes 1..6, heading.level
    assert_instance_of Anydoc::Inline, heading.content.first
    assert_instance_of String, heading.content.first.text
    assert_includes [true, false], heading.content.first.style.bold
  end

  def test_to_document_carries_embedded_assets_as_binary_strings
    document = Anydoc.to_document(File.binread(RICH), "docx")
    image = document.assets.find { _1.media_type == "image/png" }

    assert_instance_of Anydoc::Asset, image
    assert_equal Encoding::BINARY, image.data.encoding
    refute_empty image.data
    assert_equal document.assets.index(image), image.id
  end

  def test_format_detection_reads_content_extension_and_path
    assert_equal :docx, Anydoc.format_from_bytes(File.binread(RICH))
    assert_nil Anydoc.format_from_bytes(File.binread(CSV))
    assert_equal :pptx, Anydoc.format_from_extension(".pptm")
    assert_equal :xlsx, Anydoc.format_from_extension("xls")
    assert_equal :odt, Anydoc.format_from_path(Pathname("report.odt"))
    assert_nil Anydoc.format_from_path("report.unknown")
  end

  def test_conversion_errors_use_specific_exception_types
    error = assert_raises(Anydoc::ConvertError) do
      Anydoc.to_markdown_bytes("not a document", :docx)
    end
    assert_match(/malformed|unsupported/, error.message)

    assert_raises(ArgumentError) { Anydoc.to_markdown_bytes("", :wat) }
    assert_raises(Errno::ENOENT) { Anydoc.to_markdown("no-such-file.docx") }
  end
end
