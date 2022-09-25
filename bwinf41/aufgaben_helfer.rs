use glob::glob;
use std::fs::read_to_string;
use std::time::Instant;

fn get_aufgaben() -> impl Iterator<Item = (String, String)> {
    if std::env::args().nth(1).is_none() {
        panic!("Die auszuführenden Aufgaben müssen als Cmd-Arg gegeben werden. Glob-Patterns / mehrere Pfade sind möglich.");
    }
    std::env::args().skip(1).flat_map(|p| {
        glob(&p)
            .expect("Glob pattern fehler")
            .into_iter()
            .flatten()
            .map(|p| {
                (
                    read_to_string(&p).expect("Datei konnte nicht gelesen werden."),
                    p.to_str().unwrap_or_default().to_owned(),
                )
            })
    })
}

pub fn loese_aufgabe(loeser: impl Fn(String)) {
    for (teilaufgabe, name) in get_aufgaben() {
        println!(r#""{name}":"#);
        let start = Instant::now();
        loeser(teilaufgabe);
        println!("Ausführungsdauer: {}ms", start.elapsed().as_millis());
        println!();
    }
}
