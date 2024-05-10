# Python-based implementation (OLD, DEPRECATED, FOR REFERENCE ONLY)

This is the old Python-based implementation and some example configuration
files. It's here for reference only. The new Rust-based implementation was
tested to produce the exact same output as `gen.py` for the same files in the
`sets` subdirectory.

## Contents

`lib` contains a fixed version of PyYAML that is known to work.

`sets` contains example input _set_ files

`schemes.yaml` contains the configuration for which hosts to sync. The stanzas
therein refer to the _set_ files in the `sets` directory.

`gen.py` is the main script that generates the output.

`Makefile` has a `gen.old` target and a `gen.new` target, in addition to a
`clean` target. Running `make gen.old` will generate the output using `gen.py`.
Subsequently running `make gen.new` will generate the output using the new
Rust-based implementation. Taking a snapshot of the output and comparing the
results should show that they are identical.
