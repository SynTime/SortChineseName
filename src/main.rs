use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

#[derive(Deserialize)]
struct JsonData {
    word: String,
    order: String,
}

fn main() -> io::Result<()> {
    let word_dict = load_word_dict("data.json")?;

    let compound_surnames_set = load_compound_surnames_set("compound_surnames.txt")?;

    let mut names = load_names("names.txt")?;
    names.reverse(); 

    names.sort_by(|a, b| compare_names(a, b, &compound_surnames_set, &word_dict));
    
    write_output("out.txt", &names)?;
    
    Ok(())
}

fn load_word_dict<P: AsRef<Path>>(path: P) -> io::Result<HashMap<String, String>> {
    let file = File::open(path)?;
    let data: Vec<JsonData> = serde_json::from_reader(file)?;
    Ok(data.into_iter()
        .map(|d| (d.word, d.order))
        .collect())
}

fn load_compound_surnames_set<P: AsRef<Path>>(path: P) -> io::Result<HashSet<String>> {
    BufReader::new(File::open(path)?)
        .lines()
        .map(|line| Ok(line?.trim().to_string()))
        .collect()
}

fn load_names<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
    BufReader::new(File::open(path)?)
        .lines()
        .map(|line| Ok(line?.trim().to_string()))
        .filter(|s| match s {
            Ok(s) => !s.is_empty(),
            _ => true
        })
        .collect()
}

fn split_name(name: &str, compound_surnames_set: &HashSet<String>) -> (String, String) {
    if name.chars().count() >= 2 {
        let mut chars = name.chars();
        let first_char = chars.next().unwrap();
        let second_char = chars.next().unwrap();
        let possible_compound_surnames = format!("{}{}", first_char, second_char);
        if compound_surnames_set.contains(&possible_compound_surnames) {
            return (possible_compound_surnames, chars.as_str().to_string());
        }
    }
    let mut chars = name.chars();
    let surname = chars.next().unwrap().to_string();
    let given_name = chars.as_str().to_string();
    (surname, given_name)
}

fn compare_chars(a: &str, b: &str, dict: &HashMap<String, String>) -> std::cmp::Ordering {
    for (c1, c2) in a.chars().zip(b.chars()) {
        let c1_str = c1.to_string();
        let c2_str = c2.to_string();

        let default_code = "66666".to_string();

        let code1 = dict.get(&c1_str).unwrap_or(&default_code);
        let code2 = dict.get(&c2_str).unwrap_or(&default_code);

        match code1.len().cmp(&code2.len()) {
            std::cmp::Ordering::Equal => {
                match code1.cmp(code2) {
                    std::cmp::Ordering::Equal => continue,
                    ord => return ord,
                }
            }
            ord => return ord,
        }
    }

    a.len().cmp(&b.len())
}

fn compare_names(
    a: &str,
    b: &str,
    compound_surnames_set: &HashSet<String>,
    word_dict: &HashMap<String, String>,
) -> std::cmp::Ordering {
    let (surname_a, given_a) = split_name(a, compound_surnames_set);
    let (surname_b, given_b) = split_name(b, compound_surnames_set);
    
    match compare_chars(&surname_a, &surname_b, word_dict) {
        std::cmp::Ordering::Equal => {
            compare_chars(&given_a, &given_b, word_dict)
        }
        ord => ord,
    }
}

fn write_output<P: AsRef<Path>>(path: P, names: &[String]) -> io::Result<()> {
    let mut file = File::create(path)?;
    for name in names {
        write!(file, "{} ", name)?;
    }
    Ok(())
}
