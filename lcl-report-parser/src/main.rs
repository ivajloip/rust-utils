#[macro_use]
extern crate clap;
use clap::App;

extern crate regex;
use regex::Regex;

extern crate yaml_rust;
use yaml_rust::YamlLoader;

use std::fs::File;
use std::io::prelude::*;

use std::collections::HashMap;

fn main() -> std::io::Result<()> {
    // The YAML file is found relative to the current file, similar to how modules are found
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let auto_tags = auto_tags(&matches)?;

    if let Some(sub) = matches.subcommand_matches("txt-to-csv") {
        txt_to_csv(sub.value_of("config").unwrap(), auto_tags)
    } else {
        Ok(())
    }
}

fn auto_tags(matcher: &clap::ArgMatches) -> Result<HashMap<String, String>, std::io::Error> {
    let mut res: HashMap<String, String> = HashMap::new();

    if let Some(filename) = matcher.value_of("auto-tag-file") {
        let mut file = File::open(filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let docs = YamlLoader::load_from_str(&contents).unwrap();
        let hash = docs[0].as_hash().unwrap();
        for (k, v) in hash.iter() {
            res.insert(k.as_str().unwrap().to_string(), v.as_str().unwrap().to_string());
        }
    }

    return Ok(res)
}

fn txt_to_csv(filename: &str, auto_tags: HashMap<String, String>) -> std::io::Result<()> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let re_line_match = Regex::new(r"^ \d{2}\.\d{2}").unwrap();
    let re_delimiter = Regex::new(r"\s*;\s*").unwrap();
    let lines = contents.lines().
        skip_while(|l| !l.contains("ANCIEN SOLDE")).
        skip(1).
        take_while(|l| !l.contains("TOTAUX")).
        filter(|l| re_line_match.is_match(l)).
        map(|l| {
            let mut chars = l.chars().collect::<Vec<char>>();
            chars[8] = ';';
            chars[9] = '"';
            chars[79] = '"';
            chars[80] = ';';
            chars[80] = ';';
            chars[105] = ';';
            if l.len() > 125 {
                chars[125] = ';';
            } else {
                chars.push(';');
            }
            chars.push(';');
            for (k, v) in auto_tags.iter() {
                if l.contains(k) {
                    for c in v.chars() {
                        chars.push(c);
                    }
                }
            }
            chars.push(';');
            re_delimiter.replace_all(&chars.into_iter().
                                     collect::<String>().
                                     replace(',', "."),
                                     ";").
                replace(";.;", ";;")
        }).
        collect::<Vec<String>>();
    println!("Date;Label;Date2;Debit;Credit;Tags;");
    println!("{}", lines.join("\n"));

    return Ok(())
}
