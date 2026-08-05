//! The document model, converted eagerly into immutable Ruby `Data` objects.

use anydoc_core::model;
use magnus::{Error, RArray, RClass, RModule, RString, Ruby, Value, prelude::*};

fn class(ruby: &Ruby, name: &str) -> Result<RClass, Error> {
    let module: RModule = ruby.class_object().const_get("Anydoc")?;
    module.const_get(name)
}

fn array(ruby: &Ruby, values: impl Iterator<Item = Result<Value, Error>>) -> Result<RArray, Error> {
    let values = values.collect::<Result<Vec<_>, _>>()?;
    Ok(ruby.ary_new_from_values(&values))
}

fn blocks(ruby: &Ruby, items: Vec<model::Block>) -> Result<RArray, Error> {
    array(ruby, items.into_iter().map(|item| block(ruby, item)))
}

fn inlines(ruby: &Ruby, items: Vec<model::Inline>) -> Result<RArray, Error> {
    array(ruby, items.into_iter().map(|item| inline(ruby, item)))
}

fn block(ruby: &Ruby, block: model::Block) -> Result<Value, Error> {
    let (kind, level, anchor, content, list, table, inner, lang, text) = match block {
        model::Block::Heading { level, anchor, content } => (
            "heading",
            Some(level),
            anchor,
            Some(inlines(ruby, content)?),
            None,
            None,
            None,
            None,
            None,
        ),
        model::Block::Paragraph(content) => {
            ("paragraph", None, None, Some(inlines(ruby, content)?), None, None, None, None, None)
        }
        model::Block::List(value) => {
            ("list", None, None, None, Some(list_value(ruby, value)?), None, None, None, None)
        }
        model::Block::Table(value) => {
            ("table", None, None, None, None, Some(table_value(ruby, value)?), None, None, None)
        }
        model::Block::BlockQuote(value) => {
            ("block_quote", None, None, None, None, None, Some(blocks(ruby, value)?), None, None)
        }
        model::Block::CodeBlock { lang, text } => {
            ("code_block", None, None, None, None, None, None, lang, Some(text))
        }
        model::Block::Rule => ("rule", None, None, None, None, None, None, None, None),
    };
    class(ruby, "Block")?
        .funcall("new", (kind, level, anchor, content, list, table, inner, lang, text))
}

fn inline(ruby: &Ruby, inline: model::Inline) -> Result<Value, Error> {
    let (kind, text, style, content, target, alt, source, anchor, note_id) = match inline {
        model::Inline::Text { text, style } => (
            "text",
            Some(text),
            Some(style_value(ruby, style)?),
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        model::Inline::Link { content, target } => (
            "link",
            None,
            None,
            Some(inlines(ruby, content)?),
            Some(link_target(ruby, target)?),
            None,
            None,
            None,
            None,
        ),
        model::Inline::Image { alt, source } => (
            "image",
            None,
            None,
            None,
            None,
            Some(alt),
            Some(image_source(ruby, source)?),
            None,
            None,
        ),
        model::Inline::Anchor(id) => ("anchor", None, None, None, None, None, None, Some(id), None),
        model::Inline::NoteRef(id) => {
            ("note_ref", None, None, None, None, None, None, None, Some(id))
        }
        model::Inline::LineBreak => ("line_break", None, None, None, None, None, None, None, None),
    };
    class(ruby, "Inline")?
        .funcall("new", (kind, text, style, content, target, alt, source, anchor, note_id))
}

fn style_value(ruby: &Ruby, style: model::Style) -> Result<Value, Error> {
    class(ruby, "Style")?.funcall("new", (style.bold, style.italic, style.strike, style.code))
}

fn link_target(ruby: &Ruby, target: model::LinkTarget) -> Result<Value, Error> {
    let (kind, value) = match target {
        model::LinkTarget::External(value) => ("external", value),
        model::LinkTarget::Relative(value) => ("relative", value),
        model::LinkTarget::Anchor(value) => ("anchor", value),
    };
    class(ruby, "LinkTarget")?.funcall("new", (kind, value))
}

fn image_source(ruby: &Ruby, source: model::ImageSource) -> Result<Value, Error> {
    let (kind, url, asset_id) = match source {
        model::ImageSource::External(url) => ("external", Some(url), None),
        model::ImageSource::Asset(id) => ("asset", None, Some(id.0)),
        model::ImageSource::Unavailable => ("unavailable", None, None),
    };
    class(ruby, "ImageSource")?.funcall("new", (kind, url, asset_id))
}

fn list_value(ruby: &Ruby, list: model::List) -> Result<Value, Error> {
    let marker = match list.marker {
        model::MarkerKind::Bullet => "bullet",
        model::MarkerKind::Decimal => "decimal",
        model::MarkerKind::LowerAlpha => "lower_alpha",
        model::MarkerKind::UpperAlpha => "upper_alpha",
        model::MarkerKind::LowerRoman => "lower_roman",
        model::MarkerKind::UpperRoman => "upper_roman",
    };
    let items = array(ruby, list.items.into_iter().map(|item| list_item(ruby, item)))?;
    class(ruby, "List")?.funcall("new", (marker, list.start, items))
}

fn list_item(ruby: &Ruby, item: model::ListItem) -> Result<Value, Error> {
    class(ruby, "ListItem")?
        .funcall("new", (blocks(ruby, item.blocks)?, item.checked, item.marker_label))
}

fn table_value(ruby: &Ruby, table: model::Table) -> Result<Value, Error> {
    let rows = table.grid.into_iter().map(|row| {
        let row = array(ruby, row.into_iter().map(|slot| cell_slot(ruby, slot)))?;
        Ok(row.as_value())
    });
    let grid = array(ruby, rows)?;
    let kind = match table.kind {
        model::TableKind::Data => "data",
        model::TableKind::Layout => "layout",
    };
    class(ruby, "Table")?.funcall("new", (grid, table.header_rows, kind))
}

fn cell_slot(ruby: &Ruby, slot: model::CellSlot) -> Result<Value, Error> {
    let (kind, cell, origin_row, origin_col) = match slot {
        model::CellSlot::Origin(value) => ("origin", Some(cell(ruby, value)?), None, None),
        model::CellSlot::Covered { origin_row, origin_col } => {
            ("covered", None, Some(origin_row), Some(origin_col))
        }
    };
    class(ruby, "CellSlot")?.funcall("new", (kind, cell, origin_row, origin_col))
}

fn cell(ruby: &Ruby, cell: model::Cell) -> Result<Value, Error> {
    class(ruby, "Cell")?.funcall("new", (blocks(ruby, cell.blocks)?, cell.col_span, cell.row_span))
}

fn note(ruby: &Ruby, note: model::Note) -> Result<Value, Error> {
    let kind = match note.kind {
        model::NoteKind::Footnote => "footnote",
        model::NoteKind::Endnote => "endnote",
    };
    class(ruby, "Note")?.funcall("new", (note.id, kind, blocks(ruby, note.blocks)?))
}

fn asset(ruby: &Ruby, asset: model::Asset) -> Result<Value, Error> {
    let data: RString = ruby.str_from_slice(&asset.bytes);
    class(ruby, "Asset")?.funcall("new", (asset.id.0, asset.media_type, asset.origin_part, data))
}

pub fn document(ruby: &Ruby, document: model::Document) -> Result<Value, Error> {
    let notes = array(ruby, document.notes.into_iter().map(|value| note(ruby, value)))?;
    let assets = array(ruby, document.assets.into_iter().map(|value| asset(ruby, value)))?;
    class(ruby, "Document")?.funcall("new", (blocks(ruby, document.blocks)?, notes, assets))
}
