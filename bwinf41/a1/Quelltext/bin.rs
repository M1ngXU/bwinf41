use aufgaben_helfer::loese_aufgabe;
use regex::Regex;

static TEXT: &str = include_str!("text.txt");

pub fn a1(nachricht: String) {
    println!("Ergebnisse für [{nachricht}]:");
    let regex = Regex::new(&format!(
        "(?im){}",
        nachricht.replace(' ', r"\s+").replace('_', r"\w+")
    ))
    .unwrap();
    for (i, uebereinstimmung) in regex.captures_iter(TEXT).enumerate() {
        println!("{i}: {}", &uebereinstimmung[0]);
    }
}

fn main() {
    loese_aufgabe(a1);
}
