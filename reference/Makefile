.PHONY: all
all: clean gen.new

.PHONY: gen.old
gen.old:
	PYTHONPATH=$(CURDIR) python3 gen.py < schemes.yaml

.PHONY: gen.new
gen.new:
	cargo run --release < schemes.yaml

.PHONY: clean
clean:
	@$(RM) $(wildcard *.prf)
