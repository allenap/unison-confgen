use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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
    let hostname = hostname
        .split_once('.')
        .map_or(hostname.as_str(), |(part, _)| part);

    // Load the schemes we can sync.
    let mut config: Config = {
        let stdin = std::io::stdin();
        serde_yaml::from_reader(stdin)?
    };

    let here_config = config
        .hosts
        .remove(hostname)
        .ok_or(format!("Configuration for {:?} was not found.", hostname))?;
    let there_configs = config.hosts;

    let peer_configs = {
        let mut configs = peer_configs(&here_config, &there_configs).collect::<Vec<_>>();
        configs.sort_by_key(|(name, _)| name.as_str());
        configs
    };

    let mut key: u8 = 1;
    for (_there_name, there_config) in peer_configs {
        let peer_sets = {
            let mut sets = here_config
                .sets
                .intersection(&there_config.sets)
                .map(String::as_str)
                .collect::<Vec<_>>();
            sets.sort();
            sets
        };

        let (name, mut lines) = make_config(&here_config, there_config, &peer_sets)?;

        if (1..=9).contains(&key) {
            lines.push("".to_string());
            lines.push(format!("key = {key}"));
            key += 1;
        }

        let path = PathBuf::from(format!("{}.prf", name));
        let mut file = std::fs::File::create(&path)?;
        writeln!(file, "# This was AUTOMATICALLY GENERATED - Do NOT edit!")?;
        writeln!(file)?;
        writeln!(file, "{}", lines.join("\n"))?;
        writeln!(file)?;
        writeln!(file, "# End")?;
    }

    Ok(())
}

fn peer_configs<'a>(
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

fn make_config(here: &Host, there: &Host, sets: &[&str]) -> Result<(String, Vec<String>)> {
    let mut lines = Vec::new();

    include(&mut lines, &[], sets)?;

    lines.push("# Don't use domain when deriving archive names.".to_string());
    lines.push(format!("clientHostName = {}", here.hostname));
    lines.push("".to_string());

    lines.push(format!("root = {}", here.home.display()));
    lines.push(format!(
        "root = ssh://{}/{}",
        there.hostname,
        there.home.display()
    ));
    lines.push("".to_string());
    lines.push(format!(
        "logfile = {}/.logs/unison.{}-{}.log",
        here.home.display(),
        here.hostname,
        there.hostname
    ));

    let extra = here
        .client
        .iter()
        .chain(there.server.iter())
        .collect::<Vec<_>>();

    if !extra.is_empty() {
        lines.push("".to_string());
        lines.push("# Extra configuration.".to_string());
        lines.extend(extra.iter().map(|line| line.to_string()));
    }

    Ok((format!("{}-{}", here.hostname, there.hostname), lines))
}

fn include(lines: &mut Vec<String>, parents: &[&str], names: &[&str]) -> Result<()> {
    for name in names {
        let path = PathBuf::from("sets").join(name);
        let setlines = std::fs::read_to_string(path)?;
        let setlines = setlines.trim().lines();

        let lineage = [*name]
            .iter()
            .copied()
            .chain(parents.iter().copied())
            .collect::<Vec<_>>();
        lines.push(format!("### {}", lineage.join(", from ")));

        for line in setlines {
            if line.starts_with("include") {
                let names = line.split_whitespace().skip(1).collect::<Vec<_>>();
                include(lines, &lineage, &names)?;
            } else {
                lines.push(line.to_string());
            }
        }

        lines.push(format!("### /{}", name));
        lines.push("".to_string());
    }

    Ok(())
}
