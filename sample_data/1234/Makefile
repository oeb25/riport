ORDER := index conclusion
DEPS := $(shell printf "json/%s.json " $(ORDER))
SRCS := $(shell printf "src/%s.md " $(ORDER))

out.json: $(DEPS) $(SRCS)
	pandoc $(DEPS) --katex -o out.json

json/%.json: src/%.md
	pandoc $^ -t json | ./graphviz.py | pandoc --katex -f json -o $@
