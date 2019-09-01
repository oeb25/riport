declare module "draft-js-prism" {
  import { EditorState } from "draft-js";

  type f = Draft.DraftDecorator;

  type x = typeof EditorState.createEmpty;
  type y = x extends (x: infer T) => any ? T : never;
  type z = y extends undefined | infer T ? T : never;

  // export const PrismDecorator: (x: { prism: any }) => z;
  // interface PrismDecorator {
  //   new (x: { prism: any }): z;
  // }
  // const f: z;
  // export default f;
  export default class PrismDecorator extends Draft.CompositeDecorator {
    constructor(x: { prism: any; defaultSyntax: string });
  }
}
