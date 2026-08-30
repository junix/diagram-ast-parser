mod d2;
mod dbml;
mod likec4;
mod nomnoml;
mod pikchr;
mod structurizr;
mod tree;
mod wavedrom;

use crate::{ast::Document, Format, ParseError, ParseOptions, ParseResult};

pub(crate) fn parse_document(
    format: Format,
    source: &str,
    options: &ParseOptions,
) -> ParseResult<Document> {
    match format {
        Format::Auto => Err(ParseError::new(
            Format::Auto,
            "internal error: auto format was not resolved",
            None,
            source,
        )),
        Format::Dbml => dbml::parse(source, options).map(Document::Dbml),
        Format::WaveDrom => wavedrom::parse(source).map(Document::WaveDrom),
        Format::D2 => d2::parse(source, options).map(Document::D2),
        Format::Structurizr => structurizr::parse(source, options).map(Document::Structurizr),
        Format::LikeC4 => likec4::parse(source, options).map(Document::LikeC4),
        Format::Nomnoml => nomnoml::parse(source).map(Document::Nomnoml),
        Format::Pikchr => pikchr::parse(source, options).map(Document::Pikchr),
    }
}
