use std::{
    fs::{read_dir, read_to_string},
    io,
};
use std::time::Instant;

static AUFGABEN_DIR: &str = "./a";

pub(crate) fn get_aufgaben(aufgabe: u8) -> io::Result<Vec<(String, String)>> {
    Ok(read_dir(format!("{AUFGABEN_DIR}{aufgabe}"))?
        .filter_map(|d| {
            d.ok().and_then(|datei| {
                datei
                    .file_name()
                    .into_string()
                    .ok()
                    .zip(read_to_string(datei.path()).ok())
            })
        })
        .collect())
}

pub(crate) fn loese_aufgabe(aufgabe: u8, loeser: impl Fn(String)) -> io::Result<()> {
    println!("Aufgabe {aufgabe}:");
    for (name, teilaufgabe) in get_aufgaben(aufgabe)?.into_iter() {
        println!(r#""{name}":"#);
        let start = Instant::now();
        loeser(teilaufgabe);
        println!("Duration: {}ms", start.elapsed().as_millis());
        println!();
    }
    Ok(())
}