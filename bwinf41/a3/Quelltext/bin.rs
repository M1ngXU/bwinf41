use aufgaben_helfer::loese_aufgabe;

use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
    fmt::Debug,
};

use itertools::Itertools;
use rayon::prelude::*;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Sudoku {
    feld: [[Option<u8>; 9]; 9],
}
impl From<&str> for Sudoku {
    fn from(s: &str) -> Self {
        Self {
            feld: s
                .trim_start_matches('\u{feff}') // manchmal gibt es ein BOM am Anfang eines Strings
                .split_ascii_whitespace()
                .chunks(9)
                .into_iter()
                .map(|zeile| {
                    zeile
                        .map(|zahl| zahl.parse().unwrap())
                        .map(|zahl| if zahl == 0 { None } else { Some(zahl) })
                        .collect_vec()
                        .try_into()
                        .unwrap()
                })
                .collect_vec()
                .try_into()
                .unwrap(),
        }
    }
}
impl Sudoku {
    fn parse(s: &str) -> (Self, Self) {
        let formattierte_datei = s.replace('\r', "");
        let (original, neu) = formattierte_datei // kein CRLF Problem
            .split("\n\n")
            .map(Self::from)
            .collect_tuple()
            .unwrap();
        (original, neu)
    }

    fn kopieren_und_umformen(&self, [z_b, z_1, z_2, z_3, s_b, s_1, s_2, s_3]: Umformungen) -> Self {
        let neu = &mut self.clone();
        neu.tauschen(z_b, 0, Sudoku::vertausche_zeilen_block);
        neu.tauschen(z_1, 0, Sudoku::vertausche_zeile);
        neu.tauschen(z_2, 3, Sudoku::vertausche_zeile);
        neu.tauschen(z_3, 6, Sudoku::vertausche_zeile);
        neu.tauschen(s_b, 0, Sudoku::vertausche_spalten_block);
        neu.tauschen(s_1, 0, Sudoku::vertausche_spalte);
        neu.tauschen(s_2, 3, Sudoku::vertausche_spalte);
        neu.tauschen(s_3, 6, Sudoku::vertausche_spalte);
        *neu
    }

    fn tauschen(&mut self, p: Umformung, o: usize, mut f: impl FnMut(&mut Self, usize, usize)) {
        match p {
            Umformung::RechtsRotieren => {
                // 0 1 2 => 2 0 1
                f(self, o, o + 1); // 1 0 2
                f(self, o, o + 2); // 2 0 1
            }
            Umformung::LinksRotieren => {
                // 0 1 2 => 1 2 0
                f(self, o, o + 1); // 1 0 2
                f(self, o + 1, o + 2); // 1 2 0
            }
            Umformung::Vertausche1_2 => f(self, o, o + 1),
            Umformung::Vertausche1_3 => f(self, o, o + 2),
            Umformung::Vertausche2_3 => f(self, o + 1, o + 2),
            Umformung::Keine => {}
        }
    }

    fn positionen_nach_zahl(&self) -> HashMap<u8, HashSet<(usize, usize)>> {
        let mut h = HashMap::<_, HashSet<_>>::new();
        self.feld
            .iter()
            .enumerate()
            .flat_map(|(y, zeile)| {
                zeile
                    .iter()
                    .enumerate()
                    .filter_map(move |(x, zahl)| zahl.map(|z| (x, y, z)))
            })
            .for_each(|(x, y, zahl)| {
                h.entry(zahl).or_default().insert((x, y));
            });
        h
    }
    fn aehnlich(&self, other: &Self) -> Option<[u8; 9]> {
        // die Felder der beiden Sudokus werden in einem Iterator über 81 Elementen transformiert,
        // bei dem jedes Element angibt, ob dieses Feld eine Ziffer enthält oder nicht (keine 0 == true)
        let [s, o] = [self, other].map(|s| s.feld.iter().flatten().map(Option::is_some));
        // sind beide Iteratoren identisch (=an den selben Stellen gibt es Ziffern/keine Ziffern)
        // so wird überprüft, ob es möglich ist, durch eine Umbenennung beide Sudokus "gleich zu setzen"
        // itertools hat viele nützliche Funktionen auf Iteratoren
        if itertools::equal(s, o) {
            // für beide Sudokus wird eine HashMap erstellt, die die Ziffer als Schlüssel besitzt
            // und als Wert die Positionen (0-indexed) in einem HashSet hat
            let [s, o] = [self, other].map(Self::positionen_nach_zahl);
            // sind beide HashMaps gleich lang (also beide haben dieselbe Anzahl an verschiedenen Ziffern),
            // so werden deren HashSets auf Gleichheit überprüft und die Umbenennung gespeichert
            if s.len() == o.len() {
                let mut umbenennung = [0; 9];
                s.into_iter()
                    // hier wird für jede Ziffer x im 1. Sudoku eine Ziffer y im 2. Sudoku gesucht,
                    // deren Positionen identisch sind
                    .map(|(n, e)| o.iter().find(|(_, f)| &&e == f).map(|(m, _)| (n, *m)))
                    .try_for_each(|o| {
                        // gibt es eine solche Ziffer x und y, so ist o = Some(x, y). x und y werden in umbenennung gespeichert.
                        // Ist dies nicht der Fall, so ist o = None und "try_for_each" gibt None zurück
                        umbenennung[o?.0 as usize - 1] = o?.1;
                        Some(())
                    })
                    // ist das Ergebnis von "try_for_each" nicht None (also es gibt eine Umbenennung),
                    // so wird die Umbenennung zurückgegeben (also statt Some(()) => Some(umbenennung))
                    .map(|_| umbenennung)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn rotiere(&mut self) {
        // = transpose + horizontal spiegeln
        let feld = self.feld; // wird kopiert
        for (y, zeile) in self.feld.iter_mut().enumerate() {
            for (x, zelle) in zeile.iter_mut().enumerate() {
                *zelle = feld[x][8 - y];
            }
        }
    }

    fn vertausche_zeile(&mut self, a: usize, b: usize) {
        if a != b {
            self.feld.swap(a, b);
        }
    }

    fn vertausche_spalte(&mut self, a: usize, b: usize) {
        if a != b {
            self.feld.iter_mut().for_each(|zeile| zeile.swap(a, b));
        }
    }

    fn vertausche_zeilen_block(&mut self, a: usize, b: usize) {
        if a != b {
            for i in 0..3 {
                self.vertausche_zeile(3 * a + i, 3 * b + i);
            }
        }
    }

    fn vertausche_spalten_block(&mut self, a: usize, b: usize) {
        if a != b {
            self.feld.iter_mut().for_each(|zeile| {
                zeile.swap(3 * a, 3 * b);
                zeile.swap(3 * a + 1, 3 * b + 1);
                zeile.swap(3 * a + 2, 3 * b + 2);
            });
        }
    }
}

fn ergebnis_wenn_anders(f: &str, p: Umformung) -> Option<String> {
    if p == Umformung::Keine {
        None
    } else {
        Some(format!("{f}{p}"))
    }
}

fn formattiere_ergebnis(
    rotiert: bool,
    [z_b, z_1, z_2, z_3, s_b, s_1, s_2, s_3]: Umformungen,
    umbenennung: [u8; 9],
) -> String {
    [
        ergebnis_wenn_anders("Zeilenblöcke:  ", z_b),
        ergebnis_wenn_anders("Zeilen  1-3:   ", z_1),
        ergebnis_wenn_anders("Zeilen  4-6:   ", z_2),
        ergebnis_wenn_anders("Zeilen  7-9:   ", z_3),
        ergebnis_wenn_anders("Spaltenblöcke: ", s_b),
        ergebnis_wenn_anders("Spalten 1-3:   ", s_1),
        ergebnis_wenn_anders("Spalten 4-6:   ", s_2),
        ergebnis_wenn_anders("Spalten 7-9:   ", s_3),
        umbenennung
            .iter()
            .enumerate()
            .any(|(i, n)| i + 1 != *n as usize)
            .then(|| format!("Umbenennung:   {}", umbenennung.into_iter().join(" "))),
        rotiert.then_some("90 Grad im Uhrzeigersinn rotiert.".to_string()),
    ]
    .into_iter()
    .flatten()
    .join("\n")
}

macro_rules! count {
    () => {0};
    ($t:tt $($r:tt)*) => {1 + count!($($r)*)};
}
macro_rules! umformung {
    ($enum:ident { $($name:ident: $str:literal),* }) => {
        #[derive(Eq, PartialEq, Copy, Clone, Debug)]
        enum $enum {
            $($name),*
        }
        const PERMUTATIONS: [$enum; count!($($name)*)] = [$($enum::$name),*];

        impl std::fmt::Display for $enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    $($enum::$name => $str),*
                })
            }
        }
    };
}

umformung! {
    Umformung {
        Keine: "1 2 3",
        LinksRotieren: "2 3 1",
        RechtsRotieren: "3 1 2",
        Vertausche1_2: "2 1 3",
        Vertausche1_3: "3 2 1",
        Vertausche2_3: "1 3 2"
    }
}
type Umformungen = [Umformung; 8];
fn get_moeglichkeiten() -> impl ParallelIterator<Item = Umformungen> {
    PERMUTATIONS
        .into_iter()
        .cartesian_product(PERMUTATIONS)
        .cartesian_product(PERMUTATIONS)
        .cartesian_product(PERMUTATIONS)
        .cartesian_product(PERMUTATIONS)
        .cartesian_product(PERMUTATIONS)
        .cartesian_product(PERMUTATIONS)
        .cartesian_product(PERMUTATIONS)
        .par_bridge()
        .into_par_iter()
        .map(|(((((((a, b), c), d), e), f), g), h)| [a, b, c, d, e, f, g, h])
}

pub fn a3(sudokus: String) {
    let (original, neu) = Sudoku::parse(&sudokus);
    let mut neu_rotiert = neu; // neu wird kopiert
    neu_rotiert.rotiere();
    println!(
        "{}",
        // alle Kombinationen von Möglichkeiten, die Zeilen-/Spalten-(Blöcke) zu permutieren, 6 ^ 8
        // dies ist ein paralleler Iterator von `rayon`, welche die Parallelisierung übernimmt
        get_moeglichkeiten()
            // suche nach der ersten Permutation, durch die das alte Sudoku "das neue wird"
            .find_map_any(|umformungen| {
                // das alte Sudoku wird kopiert und danach umgeformt
                let neue_moeglichkeit = original.kopieren_und_umformen(umformungen);
                // ist das neu-erzeugte Sudoku einem der neuen Sudoku "ähnlich"
                // (=alles identisch ausser eine Umbenennung der Ziffern)
                let passend = neue_moeglichkeit.aehnlich(&neu);
                let nach_rotation_passend = neue_moeglichkeit.aehnlich(&neu_rotiert);
                // ist eines der beiden "ähnlich", so wird das Ergebnis formattiert und zurückgegeben
                passend.or(nach_rotation_passend).map(|umbenennung| {
                    formattiere_ergebnis(nach_rotation_passend.is_some(), umformungen, umbenennung)
                })
            })
            .unwrap_or_else(|| "Unterschiedliche Sudokus!".to_string())
    );
}

fn main() {
    loese_aufgabe(a3);
}
