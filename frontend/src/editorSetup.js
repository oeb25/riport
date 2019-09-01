import { languages } from "monaco-editor";
// import * as markdown from "monaco-editor/esm/vs/basic-languages/markdown/markdown";
// languages.register(markdown);
monaco.editor.defineTheme("solarized-dark", {
  base: "vs-dark",
  inherit: true,
  rules: [
    // Scraped from https://github.com/braver/vscode-solarized/
    // { token: "", foreground: "839496", background: "002b36" },
    { token: "", foreground: "ffffff", background: "002b36" },
    { token: "comment", foreground: "586e75" },
    { token: "meta.documentation", foreground: "586e75" },
    { token: "string", foreground: "2aa198" },
    { token: "string.regexp", foreground: "2aa198" },
    { token: "constant.character.escape", foreground: "dc322f" },
    { token: "constant.numeric", foreground: "6c71c4" },
    { token: "variable", foreground: "268bd2" },
    { token: "variable.other.readwrite", foreground: "839496" },
    { token: "variable.other.object", foreground: "839496" },
    { token: "variable.other.constant", foreground: "839496" },
    { token: "variable.function", foreground: "b58900" },
    { token: "variable.language.this", foreground: "d33682" },
    { token: "variable.language.super", foreground: "d33682" },
    { token: "keyword", foreground: "859900" },
    { token: "meta.import keyword", foreground: "cb4b16" },
    { token: "keyword.control.import", foreground: "cb4b16" },
    { token: "keyword.control.import.from", foreground: "cb4b16" },
    { token: "keyword.other.import", foreground: "cb4b16" },
    { token: "keyword.control.at-rule.include", foreground: "cb4b16" },
    { token: "keyword.control.at-rule.import", foreground: "cb4b16" },
    { token: "keyword.operator.comparison", foreground: "657b83" },
    { token: "keyword.operator.assignment", foreground: "657b83" },
    { token: "keyword.operator.arithmetic", foreground: "657b83" },
    { token: "storage", foreground: "859900" },
    { token: "keyword.control.class", foreground: "b58900" },
    { token: "meta.class", foreground: "b58900" },
    { token: "entity.name.class", foreground: "b58900" },
    { token: "entity.name.type.class", foreground: "b58900" },
    { token: "support.type", foreground: "859900" },
    { token: "support.class", foreground: "859900" },
    { token: "entity.name.function", foreground: "b58900" },
    { token: "punctuation.definition.variable", foreground: "859900" },
    { token: "constant.language", foreground: "b58900" },
    { token: "meta.preprocessor", foreground: "b58900" },
    { token: "support.function.construct", foreground: "dc322f" },
    { token: "keyword.other.new", foreground: "dc322f" },
    { token: "constant.character", foreground: "cb4b16" },
    { token: "constant.other", foreground: "cb4b16" },
    { token: "entity.name.tag", foreground: "268bd2" },
    { token: "punctuation.definition.tag.html", foreground: "586e75" },
    { token: "punctuation.definition.tag.begin", foreground: "586e75" },
    { token: "punctuation.definition.tag.end", foreground: "586e75" },
    { token: "support.function", foreground: "859900" },
    { token: "punctuation.separator.continuation", foreground: "dc322f" },
    { token: "storage.type", foreground: "268bd2" },
    { token: "support.type.exception", foreground: "cb4b16" },
    { token: "keyword.other.special-method", foreground: "cb4b16" },
    { token: "invalid", background: "6e2e32" },
    { token: "string.quoted.double", foreground: "2aa198" },
    { token: "string.quoted.single", foreground: "2aa198" },
    { token: "punctuation.definition.string.begin", foreground: "839496" },
    { token: "punctuation.definition.string.end", foreground: "839496" },
    { token: "meta.brace.square", foreground: "268bd2" },
    { token: "meta.brace.round", foreground: "657b83" },
    { token: "punctuation.definition.parameters.begin", foreground: "657b83" },
    { token: "punctuation.definition.parameters.end", foreground: "657b83" },
    { token: "meta.brace.curly", foreground: "657b83" },
    { token: "support.constant.color", foreground: "b58900" },
    {
      token: "invalid.deprecated.color.w3c-non-standard-color-name.scss",
      foreground: "b58900"
    },
    { token: "meta.selector.css", foreground: "657b83" },
    { token: "entity.name.tag.css", foreground: "b58900" },
    { token: "entity.name.tag.scss", foreground: "b58900" },
    {
      token: "source.less keyword.control.html.elements",
      foreground: "b58900"
    },
    { token: "source.sass keyword.control.untitled", foreground: "b58900" },
    { token: "entity.other.attribute-name.class.css", foreground: "b58900" },
    { token: "entity.other.attribute-name.class.sass", foreground: "b58900" },
    {
      token: "source.css entity.other.attribute-name.id",
      foreground: "b58900"
    },
    {
      token: "source.less entity.other.attribute-name.id",
      foreground: "b58900"
    },
    {
      token: "source.scss entity.other.attribute-name.id",
      foreground: "b58900"
    },
    {
      token: "source.sass entity.other.attribute-name.id",
      foreground: "b58900"
    },
    {
      token: "entity.other.attribute-name.pseudo-element.css",
      foreground: "268bd2"
    },
    { token: "entity.other.attribute-name.pseudo-class", foreground: "268bd2" },
    {
      token: "entity.other.attribute-name.tag.pseudo-class",
      foreground: "268bd2"
    },
    { token: "text.html.basic meta.tag.other.html", foreground: "657b83" },
    { token: "text.html.basic meta.tag.any.html", foreground: "657b83" },
    { token: "text.html.basic meta.tag.block.any", foreground: "657b83" },
    { token: "text.html.basic meta.tag.inline.any", foreground: "657b83" },
    {
      token: "text.html.basic meta.tag.structure.any.html",
      foreground: "657b83"
    },
    { token: "text.html.basic source.js.embedded.html", foreground: "657b83" },
    { token: "punctuation.separator.key-value.html", foreground: "657b83" },
    {
      token: "text.html.basic entity.other.attribute-name.html",
      foreground: "b58900"
    },
    { token: "meta.tag.xml entity.other.attribute-name", foreground: "b58900" },
    { token: "keyword.other.special-method.ruby", foreground: "859900" },
    { token: "variable.other.constant.ruby", foreground: "b58900" },
    { token: "constant.other.symbol.ruby", foreground: "2aa198" },
    { token: "keyword.other.special-method.ruby", foreground: "cb4b16" },
    {
      token: "meta.array support.function.construct.php",
      foreground: "b58900"
    },
    { token: "entity.name.function.preprocessor.c", foreground: "cb4b16" },
    { token: "meta.preprocessor.c.include", foreground: "cb4b16" },
    { token: "meta.preprocessor.macro.c", foreground: "cb4b16" },
    { token: "meta.preprocessor.c.include string", foreground: "2aa198" },
    {
      token: "meta.preprocessor.c.include punctuation.definition.string.begin",
      foreground: "2aa198"
    },
    {
      token: "meta.preprocessor.c.include punctuation.definition.string.end",
      foreground: "2aa198"
    },
    { token: "other.package.exclude", foreground: "dc322f" },
    { token: "other.remove", foreground: "dc322f" },
    { token: "other.add", foreground: "2aa198" },
    { token: "punctuation.section.group.tex", foreground: "dc322f" },
    {
      token: "punctuation.definition.arguments.begin.latex",
      foreground: "dc322f"
    },
    {
      token: "punctuation.definition.arguments.end.latex",
      foreground: "dc322f"
    },
    { token: "punctuation.definition.arguments.latex", foreground: "dc322f" },
    { token: "meta.group.braces.tex", foreground: "b58900" },
    { token: "string.other.math.tex", foreground: "b58900" },
    { token: "variable.parameter.function.latex", foreground: "cb4b16" },
    { token: "punctuation.definition.constant.math.tex", foreground: "dc322f" },
    { token: "text.tex.latex constant.other.math.tex", foreground: "2aa198" },
    { token: "constant.other.general.math.tex", foreground: "2aa198" },
    { token: "constant.other.general.math.tex", foreground: "2aa198" },
    { token: "constant.character.math.tex", foreground: "2aa198" },
    { token: "string.other.math.tex", foreground: "b58900" },
    { token: "punctuation.definition.string.begin.tex", foreground: "dc322f" },
    { token: "punctuation.definition.string.end.tex", foreground: "dc322f" },
    { token: "keyword.control.label.latex", foreground: "2aa198" },
    {
      token: "text.tex.latex constant.other.general.math.tex",
      foreground: "2aa198"
    },
    {
      token: "variable.parameter.definition.label.latex",
      foreground: "dc322f"
    },
    { token: "support.function.be.latex", foreground: "859900" },
    { token: "support.function.section.latex", foreground: "cb4b16" },
    { token: "support.function.general.tex", foreground: "2aa198" },
    { token: "keyword.control.ref.latex", foreground: "2aa198" },
    { token: "storage.type.class.python", foreground: "859900" },
    { token: "storage.type.function.python", foreground: "859900" },
    { token: "storage.modifier.global.python", foreground: "859900" },
    { token: "support.type.exception.python", foreground: "b58900" },
    { token: "meta.scope.for-in-loop.shell", foreground: "586e75" },
    { token: "variable.other.loop.shell", foreground: "586e75" },
    { token: "meta.scope.case-block.shell", foreground: "586e75" },
    { token: "meta.scope.case-body.shell", foreground: "586e75" },
    {
      token: "punctuation.definition.logical-expression.shell",
      foreground: "dc322f"
    },
    { token: "storage.modifier.import.java", foreground: "93a1a1" },
    { token: "support.function.perl", foreground: "268bd2" },
    { token: "meta.diff", foreground: "586e75" },
    { token: "meta.diff.header", foreground: "586e75" },
    { token: "meta.diff.range", foreground: "268bd2" },
    { token: "markup.deleted", foreground: "dc322f" },
    { token: "markup.changed", foreground: "2aa198" },
    { token: "markup.inserted", foreground: "859900" },
    { token: "markup.heading", foreground: "b58900" },
    { token: "punctuation.definition.heading.markdown", foreground: "b58900" },
    { token: "markup.quote", foreground: "859900" },
    { token: "markup.italic", fontStyle: "italic" },
    { token: "markup.bold", fontStyle: "bold" },
    { token: "markup.underline.link.markdown", foreground: "2aa198" },
    {
      token: "meta.link.reference constant.other.reference.link.markdown",
      foreground: "2aa198"
    },
    { token: "constant.other.reference.link.markdown", foreground: "6c71c4" },
    {
      token: "meta.paragraph.markdown meta.dummy.line-break",
      background: "586e75"
    },
    {
      token: "sublimelinter.notes",
      background: "586e75",
      foreground: "586e75"
    },
    {
      token: "sublimelinter.outline.illegal",
      background: "586e75",
      foreground: "586e75"
    },
    { token: "sublimelinter.underline.illegal", background: "dc322f" },
    {
      token: "sublimelinter.outline.warning",
      background: "839496",
      foreground: "839496"
    },
    { token: "sublimelinter.underline.warning", background: "b58900" },
    {
      token: "sublimelinter.outline.violation",
      background: "657b83",
      foreground: "657b83"
    },
    { token: "sublimelinter.underline.violation", background: "cb4b16" },
    { token: "sublimelinter.mark.warning", foreground: "b58900" },
    { token: "sublimelinter.mark.error", foreground: "dc322f" },
    { token: "sublimelinter.gutter-mark", foreground: "657b83" },
    { token: "brackethighlighter.all", foreground: "586e75" },
    { token: "entity.name.filename.find-in-files", foreground: "2aa198" },
    {
      token: "constant.numeric.line-number.find-in-files",
      foreground: "586e75"
    },
    { token: "markup.deleted.git_gutter", foreground: "dc322f" },
    { token: "markup.inserted.git_gutter", foreground: "859900" },
    { token: "markup.changed.git_gutter", foreground: "b58900" },
    { token: "meta.class punctuation", foreground: "839496" }
  ],
  colors: {
    "editorIndentGuides.background": "#1CD1FF12",
    "editorIndentGuide.activeBackground": "#1CD1FF12",
    // "editor.background": "#002b36",
    "editor.background": "#2d3748",
    "editor.caret": "#eee8d5",
    // "editor.foreground": "#839496",
    "editor.foreground": "#ffffff",
    "editor.gutter": "#073642",
    "editor.invisibles": "#586e75",
    "editor.lineHighlight": "#1CD1FF12",
    "editor.selection": "#586e7559",
    "editor.inactiveSelection": "#586e7540",
    "editor.selectionBorder": "#586e75",
    "editor.guide": "#1CD1FF12",
    "editor.activeLinkForeground": "#268bd2",
    "editor.selectionHighlight": "#2aa19826",
    "editor.hoverHighlight": "#2aa19826",
    "editor.findMatchHighlight": "#85990033",
    "editor.currentFindMatchHighlight": "#85990059",
    "editor.wordHighlight": "#6c71c433",
    "editor.wordHighlightStrong": "#6c71c44D",
    "editor.referenceHighlight": "#6c71c466",
    "editor.rangeHighlight": "#6c71c41A",
    "editor.findRangeHighlight": "#6c71c433"
  }
});
