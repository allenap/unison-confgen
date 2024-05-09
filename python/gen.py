#!/usr/bin/env python3

from collections import namedtuple
from operator import attrgetter
from os.path import join

import yaml


def include(lines, parents, *names):
    for name in names:
        with open(join(u'sets', name), 'r') as fd:
            setlines = fd.read().strip().splitlines()

        lineage = (name,) + parents
        lines.append('### %s' % ', from '.join(lineage))

        for line in setlines:
            if line.startswith('include'):
                include(lines, lineage, *line.split()[1:])
            else:
                lines.append(line)

        lines.append('### /%s' % name)
        lines.append('')


HostConfig = namedtuple(
    "HostConfig", ("name", "hostname", "home", "sets", "client", "server"))


def read_config(fin):
    config = yaml.safe_load(fin)
    for name, host_config in config["hosts"].items():
        yield HostConfig(
            name, host_config["hostname"], host_config["home"],
            frozenset(host_config["sets"]), host_config.get("client", []),
            host_config.get("server", []))


def make_configs(here_config, there_configs):
    for there_config in sorted(there_configs, key=attrgetter("name")):
        sets = here_config.sets & there_config.sets
        if len(sets) != 0:
            extra = here_config.client + there_config.server
            yield make_config(here_config, there_config, sets, extra)


def make_config(here, there, sets, extra):
    lines = []

    include(lines, (), *sets)

    lines.append("# Don't use domain when deriving archive names.")
    lines.append('clientHostName = %s' % here.hostname)
    lines.append('')

    lines.append('root = %s' % here.home)
    lines.append('root = ssh://%s/%s' % (there.hostname, there.home))
    lines.append('')
    lines.append(
        'logfile = %s/.logs/unison.%s-%s.log' % (
            here.home, here.name, there.name))

    if len(extra) > 0:
        lines.append('')
        lines.append("# Extra configuration.")
        lines.extend(extra)

    return '%s-%s' % (here.name, there.name), lines


if __name__ == '__main__':
    from sys import argv, stdin

    if len(argv) >= 2:
        hostname = argv[1]
    else:
        import socket
        hostname = socket.gethostname().lower()

    config = {cfg.name: cfg for cfg in read_config(stdin)}
    hostname = hostname.split('.', 1)[0]  # Discard domain.

    if hostname in config:
        here_config = config.pop(hostname)
        there_configs = config.values()
    else:
        raise SystemExit("Configuration for %r was not found." % hostname)

    key = 1
    for name, lines in make_configs(here_config, there_configs):
        if 1 <= key <= 9:
            lines.append('')
            lines.append('key = %d' % key)
            key += 1

        with open(u'%s.prf' % name, 'w') as fd:
            fd.write('# This was AUTOMATICALLY GENERATED - Do NOT edit!\n\n')
            fd.write('\n'.join(lines))
            fd.write('\n\n# End\n')


# End
