# Allenap's Unison Configuration Generator

This helps me generate my [Unison](https://github.com/bcpierce00/unison)
configuration files.

## Background

Unison does bidirectional file synchronisation. I write a configuration file,
run `unison name-of-config-file`, and Unison synchronises files between _here_
and _there_. The _there_ can be another directory, or it can be another machine
reachable over SSH. This is typically how I use it.

I have a few machines that I keep in sync. I don't have a central server that
serves as a source of truth, so I'm usually syncing back and forth from a few
different machines. One day I'll run `unison` on machine A to sync to machine B,
and another day I'll do the reverse, then also to machine C, and combinations of
those. It follows that I want to have the same configuration on every machine
– so I use Unison itself to sync those configuration files around.

I also have different _sets_ of files that I want to sync. I want to sync
dotfiles up to my server in the cloud, but not my financial records. I want to
sync my coding projects between my laptop and my Linux machine under the desk,
but not my photos.

## Principle of operation

This script takes a single configuration file – read from stdin – that describes
all the hosts that I have. For example:

```toml
# schemes.toml
[hosts.alice]
hostname = "alice.example.com"
home = "/home/gavin"
sets = ["common", "financial", "github"]

[hosts.bob]
hostname = "bob.example.org"
home = "/Users/gavin"
sets = ["common", "github", "photos"]

[hosts.carol]
hostname = "carol.example.net"
home = "/home/gavin"
sets = ["common", "financial", "photos"]
```

This configuration file describes three hosts – _alice_, _bob_, and _carol_ –
and the sets of files that each syncs.

When I run `unison-confgen < schemes.toml` on _alice_, two configuration files
are generated:

- `alice-bob.prf`
- `alice-carol.prf`

These contain standard Unison configuration directives to sync between hosts
_for the sets of files that those hosts have in common_.

Now I can type `unison alice-bob` or `unison alice-carol` to:

- sync the `common` and `github` sets between _alice_ and _bob_, or
- sync the `common` and `financial` sets between _alice_ and _carol_.

Likewise, _bob_ will have `bob-alice.prf` and `bob-carol.prf`, and _carol_ will
have `carol-alice.prf` and `carol-bob.prf`.

### What's in a _set_?

A _set_ file lives in a `sets` subdirectory. It's named exactly the same as the
name of the set; there's no file extension. It contains Unison configuration
directives. It doesn't have to describe a set of files, but typically it might
look like:

```
# sets/github
path = GitHub
ignore = Regex GitHub/.*/target
ignore = Name .overmind.sock
```

Paths, like `GitHub` above, are resolved relative to the `home` setting from
`schemes.toml`.

These set files are included verbatim in the generated configuration files, with
one addition: one can use an `include` directive to include another set file:

```
# sets/all-code
include github
include gitlab
include projects
```

## Usage

I put the `unison-confgen` binary on `PATH` – typically because I've used `cargo
install` to install it, and Cargo's `bin` directory is already on my `PATH`.

Then, in `~/.unison`, I have the `etc/schemes.toml` configuration file, a `sets`
subdirectory, and a `Makefile` to drive it:

```Makefile
.PHONY: all
all: clean gen

.PHONY: gen
gen:
	@type unison-confgen >/dev/null || cargo install allenap-unison-confgen
	unison-confgen < etc/schemes.toml

.PHONY: clean
clean:
	@$(RM) $(wildcard *.prf)

.PHONY: complete
complete: clean gen
	@echo $(basename $(wildcard *.prf))
```

Running `make -C ~/.unison` clears away all `.prf` files and regenerates them.

The `complete` target is useful for shell completion. I use Bash with the
following code in my `~/.bashrc`:

```bash
# Unison stuff
# shellcheck disable=SC2317
_unison() {
    local cur words
    # The current word due for completion.
    cur=${COMP_WORDS[COMP_CWORD]}
    # Non-local array storing the possible completions.
    COMPREPLY=()
    case "${cur}" in
        -*)
            words="$(unison -help | awk '/^ -/ { print $1 }')"
            ;;
        *)
            words="$(make -sC ~/.unison complete)"
            ;;
    esac
    mapfile -t COMPREPLY < <(compgen -W "${words}" -- "${cur}")
    return 0
}

for _unison in $(compgen -c unison)
do
    complete -F _unison "$_unison"
done
unset _unison

# Override local hostname. Useful when WiFi publishes a different
# local domain (<cough>FritzBox</cough>), for example.
if [ -f ~/.unison/hostname ]
then
    read -r UNISONLOCALHOSTNAME < ~/.unison/hostname &&
        export UNISONLOCALHOSTNAME
fi
```

That's all. It's simple but effective; I've been using it for years. Recently I
rewrote it in Rust – from Python – and this is the first time I've published it.
Since it does all I need, I probably won't work much on it. Well, maybe a rainy
day will see me:

- Add a proper command-line parser;
- Move the shell completion code into the Rust binary;
- Eliminate the need for that `Makefile`.

But I'm not holding my breath.

Have fun!

## License

[GNU General Public License 3.0](https://www.gnu.org/licenses/gpl-3.0.html) (or
later). See [LICENSE](LICENSE).
