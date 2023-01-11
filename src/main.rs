use std::{process::Command, path::Path, fs::File};

fn main() {
    // Cargo wird ausgeführt und die .exe Dateien in die aufgaben kopiert
    println!("Release mode: {}", cfg!(not(debug_assertions)));
    let mut command = &mut Command::new("cargo");
    command = command.args(["build", "--package", "bwinf41-1", "--bin", "a[135]"]);
    if cfg!(not(debug_assertions)) {
        command = command.arg("--release");
    }
    assert!(command.spawn().expect("Cargo konnte nicht gestartet werden.").wait().expect("Cargo konnte nicht erfolgreich gelaufen werden.").success(), "Cargo ist nicht erfolgreich beendet.");
    let path = Path::new("target").join(if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    });
    for aufgabe in ["a1", "a3", "a5"] {
        let mut exe = path.clone();
        let mut bin = Path::new("bwinf41-1").join(aufgabe).join("AusfuehrbaresProgramm");
        std::fs::create_dir_all(&bin).expect("Der Ordner `AusfuehrbaresProgramm` konnte nicht erstellt werden.");
        for pfad in [&mut exe, &mut bin] {
            pfad.push(aufgabe);
            pfad.set_extension(std::env::consts::EXE_EXTENSION);
        }
        let size = exe.metadata().expect("Metadaten konnten nicht gelesen werden.").len();
        assert_eq!(size, std::fs::copy(exe, bin).expect("Die ausführbare Datei konnte nicht kopiert werden."), "Die ausführbare Datei konnte nicht vollständig kopiert werden.");
        println!("`{}` wurde erfolgreich kopiert.", aufgabe);
    }
    print!("`bwinf41-1` wird gezippt...");
    zip_dir::zip_dir(Path::new("bwinf41-1"), File::create("bwinf41-1.zip").expect("Failed to open zip file."), None).expect("Failed to zip file.");
    println!("Fertig.");
}