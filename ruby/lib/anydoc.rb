# frozen_string_literal: true

require_relative "anydoc/version"

module Anydoc
  class Error < StandardError; end

  # Raised when an input can be read, but meaningful conversion is impossible.
  class ConvertError < Error; end

  Document = Data.define(:blocks, :notes, :assets)
  Block = Data.define(:kind, :level, :anchor, :content, :list, :table, :blocks, :lang, :text)
  Inline = Data.define(:kind, :text, :style, :content, :target, :alt, :source, :anchor, :note_id)
  Style = Data.define(:bold, :italic, :strike, :code)
  LinkTarget = Data.define(:kind, :value)
  ImageSource = Data.define(:kind, :url, :asset_id)
  List = Data.define(:marker, :start, :items)
  ListItem = Data.define(:blocks, :checked, :marker_label)
  Table = Data.define(:grid, :header_rows, :kind)
  CellSlot = Data.define(:kind, :cell, :origin_row, :origin_col)
  Cell = Data.define(:blocks, :col_span, :row_span)
  Note = Data.define(:id, :kind, :blocks)
  Asset = Data.define(:id, :media_type, :origin_part, :data)
end

require "anydoc/anydoc"

module Anydoc
  FORMATS = %i[doc docx odt pdf ppt pptx rtf epub xlsx ods odp csv].freeze

  class << self
    def format_from_bytes(data)
      _format_from_bytes(data)&.to_sym
    end

    def format_from_extension(extension)
      _format_from_extension(extension)&.to_sym
    end

    def format_from_path(path)
      _format_from_path(File.path(path))&.to_sym
    end

    def to_markdown(path)
      _to_markdown(File.path(path))
    end

    def to_markdown_bytes(data, format = nil)
      _to_markdown_bytes(data, format&.to_s)
    end

    def to_document(data, format = nil)
      _to_document(data, format&.to_s)
    end
  end
end
