use regex::Regex;

static TEXT: &str = include_str!("../text.txt");

pub fn a1(nachricht: String) {
    println!("Ergebnisse fuer [{nachricht}]:");
    let regex = Regex::new(&format!(
        "(?im){}",
        nachricht.replace(' ', r"[\s|\n]+").replace('_', r"\w+")
    ))
    .unwrap();
    for (i, uebereinstimmung) in regex.captures_iter(TEXT).enumerate() {
        println!("{i}: {}", &uebereinstimmung[0]);
    }
}
