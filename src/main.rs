use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::io::{Read, Write as IoWrite};
use std::path::PathBuf;

use itertools::chain;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Config {
    hosts: HashMap<String, Host>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(deny_unknown_fields, default)]
struct Host {
    hostname: String,
    home: PathBuf,
    sets: HashSet<String>,
    client: Vec<String>,
    server: Vec<String>,
}

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    // Get the hostname of the machine we're running on.
    let hostname = hostname::get()?.into_string().unwrap_or_default();
    // Trim the domain name from the hostname.
    let here_name = hostname
        .split_once('.')
        .map_or(hostname.as_str(), |(part, _)| part);

    // Load the schemes we can sync.
    let mut hosts = {
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        // First try to parse the input as YAML, then fall back to TOML.
        // Eventually I'll remove the YAML option.
        let config = match serde_yaml::from_str::<'_, Config>(&buffer) {
            Err(_) => toml::from_str::<Config>(&buffer)?,
            Ok(config) => config,
        };
        config.hosts
    };

    let here_config = hosts
        .remove(here_name)
        .ok_or(format!("Configuration for {here_name:?} was not found."))?;
    let there_configs = hosts;

    let pair_configs = {
        let mut configs = pair_configs(&here_config, &there_configs).collect::<Vec<_>>();
        configs.sort_by_key(|(name, _)| name.as_str());
        configs
    };

    let mut key: u8 = 1;
    for (there_name, there_config) in pair_configs {
        let pair_sets = {
            let mut sets = here_config
                .sets
                .intersection(&there_config.sets)
                .map(String::as_str)
                .collect::<Vec<_>>();
            sets.sort();
            sets
        };

        let mut config = config(
            (here_name, &here_config),
            (there_name, there_config),
            &pair_sets,
        )?;

        if (1..=9).contains(&key) {
            writeln!(config)?;
            writeln!(config, "key = {key}")?;
            key += 1;
        }

        let prf_path: PathBuf = format!("{here_name}-{there_name}.prf").into();
        let mut file = std::fs::File::create(&prf_path)?;
        writeln!(file, "# This was AUTOMATICALLY GENERATED - Do NOT edit!")?;
        writeln!(file)?;
        write!(file, "{}", config)?;
        writeln!(file)?;
        writeln!(file, "# End")?;
    }

    Ok(())
}

fn pair_configs<'a>(
    here_config: &'a Host,
    there_configs: &'a HashMap<String, Host>,
) -> impl Iterator<Item = (&'a String, &'a Host)> {
    there_configs.iter().filter(|(_, there_config)| {
        here_config
            .sets
            .intersection(&there_config.sets)
            .next()
            .is_some()
    })
}

fn config(
    (here_name, here): (&str, &Host),
    (there_name, there): (&str, &Host),
    sets: &[&str],
) -> Result<String> {
    let mut config = String::new();

    include(&mut config, &[], sets)?;

    writeln!(config, "# Don't use domain when deriving archive names.")?;
    writeln!(config, "clientHostName = {}", here.hostname)?;
    writeln!(config)?;

    writeln!(config, "root = {}", here.home.to_string_lossy())?;
    writeln!(
        config,
        "root = ssh://{}/{}",
        there.hostname,
        there.home.to_string_lossy()
    )?;
    writeln!(config)?;
    writeln!(
        config,
        "logfile = {}/.logs/unison.{here_name}-{there_name}.log",
        here.home.to_string_lossy(),
    )?;

    let extra = chain!(&here.client, &there.server).collect::<Vec<_>>();
    if !extra.is_empty() {
        writeln!(config)?;
        writeln!(config, "# Extra configuration.")?;
        for line in extra {
            writeln!(config, "{line}")?;
        }
    }

    Ok(config)
}

fn include(config: &mut String, parents: &[&str], names: &[&str]) -> Result<()> {
    for name in names {
        let path = PathBuf::from("sets").join(name);
        let set = std::fs::read_to_string(path)?;

        let lineage = chain!([name], parents).copied().collect::<Vec<_>>();
        writeln!(config, "### {}", lineage.join(", from "))?;

        for line in set.trim().lines() {
            if line.starts_with("include") {
                let names = line.split_whitespace().skip(1).collect::<Vec<_>>();
                include(config, &lineage, &names)?;
            } else {
                writeln!(config, "{line}")?;
            }
        }

        writeln!(config, "### /{name}")?;
        writeln!(config)?;
    }

    Ok(())
}
