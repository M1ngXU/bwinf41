use std::fs::read_to_string;
use std::time::Instant;
use glob::glob;

pub fn loese_aufgabe(loeser: impl Fn(String)) {
    for (teilaufgabe, name) in glob(
        &std::env::args()
            .nth(1)
            .expect("Gib ein Glob Pattern als erstes Argument an."),
    )
    .expect("Glob pattern fehler")
    .into_iter()
    .flatten()
    .map(|p| {
        (
            read_to_string(&p).expect("Datei konnte nicht gelesen werden."),
            p.to_str().unwrap_or_default().to_owned(),
        )
    }) {
        println!(r#""{name}":"#);
        let start = Instant::now();
        loeser(teilaufgabe);
        println!("Ausführungsdauer: {}ms", start.elapsed().as_millis());
        println!();
    }
}
