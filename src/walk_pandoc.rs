/*
    Pandoc(
        Meta({}),
        [
            Para([
                Str("Inline:"),
                Space,
                Math(InlineMath, "\\sum n = \\infin")
            ]),
            Para([
                Math(DisplayMath, "\n\\int_a^b 1 = b - a\n")]),
                Para([Str("end.")
            ])
        ]
    )
*/

use pandoc_types::definition::*;

pub trait Walk {
    fn block(&mut self, block: Block) -> Vec<Block> {
        vec![block]
    }
    fn inline(&mut self, inline: Inline) -> Vec<Inline> {
        vec![inline]
    }
}

pub fn walk_block<W: Walk>(walker: &mut W, block: Block) -> Vec<Block> {
    walker
        .block(block)
        .into_iter()
        .map(|block| {
            let mut walk = |inline: Vec<Inline>| {
                inline
                    .into_iter()
                    .flat_map(|inline| walk_inline(walker, inline))
                    .collect()
            };
            match block {
                Block::Plain(inline) => Block::Plain(walk(inline)),
                Block::Para(inline) => Block::Para(walk(inline)),
                _ => block,
            }
        })
        .collect()
}

pub fn walk_inline<W: Walk>(walker: &mut W, inline: Inline) -> Vec<Inline> {
    walker
        .inline(inline)
        .into_iter()
        .map(|inline| {
            let mut walk = |inline: Vec<Inline>| {
                inline
                    .into_iter()
                    .flat_map(|inline| walk_inline(walker, inline))
                    .collect()
            };
            match inline {
                Inline::Emph(inline) => Inline::Emph(walk(inline)),
                Inline::Strong(inline) => Inline::Strong(walk(inline)),
                Inline::Span(attr, inline) => Inline::Span(attr, walk(inline)),
                _ => inline,
            }
        })
        .collect()
}

pub fn walk_pandoc<W: Walk>(walker: &mut W, pandoc: Pandoc) -> Pandoc {
    Pandoc(
        pandoc.0,
        pandoc
            .1
            .into_iter()
            .flat_map(|block| walk_block(walker, block))
            .collect(),
    )
}
