use aufgaben_helfer::loese_aufgabe;
use regex::Regex;

static TEXT: &str = include_str!("text.txt");

pub fn a1(nachricht: String) {
    println!("Ergebnisse für [{nachricht}]:");
    // für die Nachricht "a _ b" wäre der reguläre Ausdruck /a\s+\w+b/i
    // \s+, da es bei Störung2 eine Lösung mit neuer Zeile + Tab gibt
    // \w+ für eine Reihe von Buchstaben
    // /i (bei dieser Regex-Bibliothek gibt man dies mit "(?i)" an), um Gross-/Kleinbuchstaben zu ignorieren
    let regex = Regex::new(&format!(
        "(?i){}",
        nachricht.replace(' ', r"\s+").replace('_', r"\w+")
    ))
    .unwrap();
    for (i, uebereinstimmung) in regex.captures_iter(TEXT).enumerate() {
        // uebereinstimmung ist wie ein Array mit dem 0ten Element als "Match"
        println!("{i}: {}", &uebereinstimmung[0]);
    }
}

fn main() {
    loese_aufgabe(a1);
}
