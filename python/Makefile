.PHONY: all
all: clean gen.new

.PHONY: gen.old
gen.old:
	PYTHONPATH=$(CURDIR) python3 gen.py < schemes.yaml

.PHONY: gen.new
gen.new:
	rust-script $(CURDIR)/../gen.rs < schemes.yaml

.PHONY: clean
clean:
	@$(RM) $(wildcard *.prf)
