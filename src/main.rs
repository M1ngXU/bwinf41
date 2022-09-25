#![deny(clippy::too_many_lines)]

use std::io;

use aufgaben_helfer::loese_aufgabe;

mod a1;
mod a3;
mod a5;
mod aufgaben_helfer;

fn main() -> io::Result<()> {
    loese_aufgabe(1, a1::a1)?;
    loese_aufgabe(3, a3::a3)?;
    loese_aufgabe(5, a5::a5)?;
    Ok(())
}
