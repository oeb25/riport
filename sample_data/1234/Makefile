ORDER := index
DEPS := $(shell printf "html/%s.html " $(ORDER))
SRCS := $(shell printf "src/%s.md " $(ORDER))

CDN := http://cdn.mathjax.org/mathjax/latest/MathJax.js?config=TeX-AMS-MML_HTMLorMML

out.html: $(DEPS) $(SRCS) header.md
	pandoc header.md $(DEPS) -s -o out.html

html/%.html: src/%.md
	pandoc $^ --highlight-style=monochrome -t json | ./graphviz.py | pandoc -f json -o $@
