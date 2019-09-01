import * as Immutable from "immutable";
import * as Prism from "prismjs";
import * as React from "react";
import * as Draft from "draft-js";
// let extend = require('extend');

/**
    Filter block to only highlight code blocks
    @param {Draft.ContentBlock}
    @return {Boolean}
*/
function defaultFilter(block: Draft.ContentBlock) {
  return block.getType() === "code-block";
}

/**
    Return syntax for highlighting a code block
    @param {Draft.ContentBlock}
    @return {String}
*/
function defaultGetSyntax(block: Draft.ContentBlock) {
  if (block.getData) {
    return block.getData().get("syntax");
  }

  return null;
}

/**
    Default render for token
    @param {Object} props
    @return {React.Element}
*/
const defaultRender: React.SFC<{ type: string }> = props => {
  return React.createElement(
    "span",
    { className: "prism-token token " + props.type },
    props.children
  );
};

type PrismOptions = {
  // Default language to use
  // defaultSyntax: string | null,
  // Filter block before highlighting
  // filter: defaultFilter,
  // Function to get syntax for a block
  // getSyntax: defaultGetSyntax,
  // Render a decorated text for a token
  // render: defaultRender,
  // Prism module
  // prism: Prism
};

let KEY_SEPARATOR = "-";

type x = typeof Draft.EditorState.createEmpty;
type y = x extends (x: infer T) => any ? T : never;
type z = y extends undefined | infer T ? T : never;

export const PrismDecorator = (): z => {
  const highlighted: { [x: string]: { [x: string]: Prism.Token } } = {};

  return {
    getDecorations: (block, contentState) => {
      let offset = 0,
        tokenCount = 0;
      let blockKey = block.getKey();
      let blockText = block.getText();
      let decorations = Array(blockText.length + 1)
        .join(",")
        .split(",")
        .map(x => (null as any) as string);

      highlighted[blockKey] = {};

      // if (!filter(block)) {
      //   return decorations;
      // }

      // let syntax = getSyntax(block) || this.options.get("defaultSyntax");
      // const syntax = "markdown";
      const syntax = block.getData().get("syntax") || "markdown";
      // console.log(syntax);

      const prev = contentState.getBlockBefore(block.getKey());
      console.log("lang", prev && prev.getData().get("lang"));

      // Allow for no syntax highlighting
      // if (syntax == null) {
      //   return Immutable.List(decorations);
      // }

      // Parse text using Prism
      let grammar = Prism.languages[syntax];
      console.log(blockText);
      let tokens = Prism.tokenize(blockText + "\n```", grammar);

      function processToken(
        decorations: string[],
        token: string | Prism.Token,
        offset: number
      ) {
        if (typeof token === "string") {
          return;
        }
        //First write this tokens full length
        const tokenId = "tok" + tokenCount++;
        const resultId = blockKey + "-" + tokenId;
        console.log({ token });
        highlighted[blockKey][tokenId] = token;
        occupySlice(decorations, offset, offset + token.length, resultId);
        //Then recurse through the child tokens, overwriting the parent
        let childOffset = offset;
        for (let i = 0; i < token.content.length; i++) {
          let childToken = (token.content as Prism.Token[])[i];
          processToken(decorations, childToken, childOffset);
          childOffset += childToken.length;
        }
      }

      for (let i = 0; i < tokens.length; i++) {
        const token = tokens[i];
        processToken(decorations, token, offset);
        offset += token.length;
      }

      return Immutable.List(decorations);
    },

    /**
     * Return component to render a decoration
     *
     * @param {String}
     * @return {Function}
     */
    getComponentForKey: (key: string) => {
      const Renderer: React.SFC<{ type: string }> = ({ type, children }) => (
        <span key={key} className={"prism-token token " + type}>
          {children}
        </span>
      );
      return Renderer;
    },

    /**
     * Return props to render a decoration
     *
     * @param {String}
     * @return {Object}
     */
    getPropsForKey: (key: string) => {
      let parts = key.split("-");
      let blockKey = parts[0];
      let tokId = parts[1];
      let token = highlighted[blockKey][tokId];

      return {
        type: token.type
      };
    }
  };
};

function occupySlice<T>(
  targetArr: T[],
  start: number,
  end: number,
  componentKey: T
) {
  for (let ii = start; ii < end; ii++) {
    targetArr[ii] = componentKey;
  }
}

// module.exports = PrismDecorator;
