import * as React from "react";
import * as katex from "katex";
import "katex/dist/katex.css";
import * as Prism from "prismjs";
import "prismjs/components/prism-python";
import "prismjs/components/prism-rust";

// import "prismjs/themes/prism-coy.css";
import "prismjs/themes/prism-dark.css";
// import "prismjs/themes/prism-funky.css";
// import "prismjs/themes/prism-okaidia.css";
// import "prismjs/themes/prism-solarizedlight.css";
// import "prismjs/themes/prism-tomorrow.css";
// import "prismjs/themes/prism-twilight.css";
// import "prismjs/themes/prism.css";

export type Document = { blocks: Fragment[] };
export type Fragment =
  | { t: "Str"; c: string }
  | { t: "Space"; c: string }
  | { t: "Header"; c: [1 | 2 | 3 | 4 | 5 | 6, [string, [], []], Fragment[]] }
  | { t: "Quoted"; c: Fragment[] }
  | { t: "SoftBreak" }
  | { t: "Emph"; c: Fragment[] }
  | { t: "Strong"; c: Fragment[] }
  | {
      t: "Link";
      c: [["", [], []], Fragment[], [string, ""]];
    }
  | { t: "Para"; c: Fragment[] }
  | {
      t: "BlockQuote";
      c: Fragment[];
    }
  | {
      t: "Image";
      c: [["", [], []], [], [string, ""]];
    }
  | { t: "Code"; c: [["", [], []], string] }
  | { t: "Math"; c: [{ t: "InlineMath" | "DisplayMath" }, string] }
  | { t: "CodeBlock"; c: [[string, string], string] };

export const Render: React.SFC<{
  projectId: string;
  src: Fragment[] | Fragment;
}> = ({ src, projectId }) => {
  const propagate = (c: Fragment[]) =>
    c.map((b, i) => <Render key={i} projectId={projectId} src={b} />);

  if (Array.isArray(src)) {
    return <>{propagate(src)}</>;
  } else if (src.t == "Para") {
    return <p>{propagate(src.c)}</p>;
  } else if (src.t == "BlockQuote") {
    return (
      <blockquote className="pl-2 border-l italic">
        {propagate(src.c)}
      </blockquote>
    );
  } else if (src.t == "Str") {
    return <>{src.c}</>;
  } else if (src.t == "Header") {
    switch (src.c[0]) {
      case 1:
        return <h1>{propagate(src.c[2])}</h1>;
      case 2:
        return <h2>{propagate(src.c[2])}</h2>;
      case 3:
        return <h3>{propagate(src.c[2])}</h3>;
      case 4:
        return <h4>{propagate(src.c[2])}</h4>;
      case 5:
        return <h5>{propagate(src.c[2])}</h5>;
      case 6:
        return <h6>{propagate(src.c[2])}</h6>;
      default:
        return (
          <p>
            h{src.c[0]}: {propagate(src.c[2])}
          </p>
        );
    }
  } else if (src.t == "Space") {
    return <> </>;
  } else if (src.t == "SoftBreak") {
    return <br />;
  } else if (src.t == "Quoted") {
    return <>"{JSON.stringify(src)}"</>;
  } else if (src.t == "Emph") {
    return <em>{propagate(src.c)}</em>;
  } else if (src.t == "Strong") {
    return <b>{propagate(src.c)}</b>;
  } else if (src.t == "Link") {
    return <a href={src.c[2][0]}>{propagate(src.c[1])}</a>;
  } else if (src.t == "Code") {
    return <code className="inline">{src.c[1]}</code>;
  } else if (src.t == "Image") {
    return (
      <span className="flex items-center justify-center m-5">
        <img src={`/api/projects/${projectId}/static/${src.c[2][0]}`} />
      </span>
    );
  } else if (src.t == "CodeBlock") {
    const lang = src.c[0][1];
    const gramma = Prism.languages[lang];
    return (
      <pre className={`code lang-${lang}`}>
        {gramma ? (
          <code
            dangerouslySetInnerHTML={{
              __html: Prism.highlight(src.c[1], gramma, lang)
            }}
          />
        ) : (
          <code>{src.c[1]}</code>
        )}
      </pre>
    );
  } else if (src.t == "Math") {
    try {
      // TODO
      if (src.c[0].t == "InlineMath") {
        return (
          <span
            dangerouslySetInnerHTML={{
              __html: katex.renderToString(src.c[1])
            }}
          />
        );
      } else {
        return (
          <span className="flex items-center justify-center">
            <span
              dangerouslySetInnerHTML={{
                __html: katex.renderToString(src.c[1], {
                  displayMode: true
                })
              }}
            />
          </span>
        );
      }
    } catch (e) {
      return (
        <>
          <br />
          KaTeX parse error: {JSON.stringify(e)}
        </>
      );
    }
  } else {
    return <>{JSON.stringify(src)}</>;
  }
};
