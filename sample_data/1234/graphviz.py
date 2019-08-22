#!/usr/bin/env python3

"""
Pandoc filter to process code blocks with class "graphviz" into
graphviz-generated images.
Needs pygraphviz
"""

import os
import sys

import pygraphviz
from subprocess import run, PIPE

from pandocfilters import toJSONFilter, Para, CodeBlock, Emph, Str, Image, get_filename4code, get_caption, get_extension


def graphviz(key, value, format, _):
    if key == 'CodeBlock':
        [[ident, classes, keyvals], code] = value
        if "graphviz" in classes:
            caption, typef, keyvals = get_caption(keyvals)
            filetype = get_extension(format, "png", html="png", latex="pdf")
            dest = get_filename4code("graphviz", code, filetype)

            if not os.path.isfile(dest):
                g = pygraphviz.AGraph(string=code)
                g.layout()
                g.draw(dest)
                sys.stderr.write('Created image ' + dest + '\n')

            return Para([Image([ident, [], keyvals], caption, [dest, typef])])
        if "python" in classes:
            p = run(['python'], stdout=PIPE,
                    input=code, encoding='ascii')

            output = "\n".join(["> " + x for x in p.stdout.split("\n") if len(x) > 0])

            return [
                CodeBlock(["", ["python"], []], code),
                CodeBlock(["", [], []], output)
            ]

if __name__ == "__main__":
    toJSONFilter(graphviz)