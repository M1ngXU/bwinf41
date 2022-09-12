use regex::Regex;

static TEXT: &'static str = include_str!("../text.txt");

pub fn a1(nachricht: String) {
    println!("Ergebnisse fuer [{nachricht}]:");
    if let Ok(regex) = Regex::new(&format!("(?i){}", nachricht.replace("_", r"\w+"))) {
        for uebereinstimmung in regex.captures_iter(TEXT) {
            println!("{}", &uebereinstimmung[0]);
        }
    } else {
        println!("Keine Treffer!");
    }
}
