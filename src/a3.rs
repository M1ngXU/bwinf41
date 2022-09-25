use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
    fmt::Debug,
};

use itertools::Itertools;
use rayon::prelude::*;

#[derive(PartialEq, Eq, Clone, Copy)]
struct Sudoku {
    feld: [[Option<u8>; 9]; 9],
}
impl Debug for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            self.feld
                .iter()
                .map(|zeile| zeile.iter().map(|n| n.unwrap_or_default()).join(" "))
                .join("\n")
        )
    }
}
impl From<&str> for Sudoku {
    fn from(s: &str) -> Self {
        let feld = s
            .trim_start_matches('\u{feff}') // BOM am Anfang eines Strings
            .split_ascii_whitespace()
            .chunks(9)
            .into_iter()
            .map(|zeile| {
                zeile
                    .map(|zahl| zahl.parse::<u8>().unwrap())
                    .map(|zahl| if zahl == 0 { None } else { Some(zahl) })
                    .collect_vec()
                    .try_into()
                    .unwrap()
            })
            .collect_vec()
            .try_into()
            .unwrap();
        Self { feld }
    }
}

impl Sudoku {
    fn aehnlich(&self, other: &Self) -> Option<[u8; 9]> {
        let [s, o] = [self, other].map(|s| s.feld.iter().flatten().map(Option::is_some));
        if itertools::equal(s, o) {
            let [s, o] = [self, other].map(|s| {
                s.feld
                    .iter()
                    .enumerate()
                    .flat_map(|(y, zeile)| {
                        zeile
                            .iter()
                            .enumerate()
                            .filter_map(|(x, n)| n.map(|n| (x, n)))
                            .map(|(x, n)| (x, y, n))
                            .collect_vec()
                    })
                    .sorted_unstable_by_key(|(_x, _y, n)| *n)
                    .group_by(|(_x, _y, n)| *n)
                    .into_iter()
                    .map(|(n, group_iter)| {
                        (
                            n,
                            group_iter.map(|(x, y, _n)| (x, y)).collect::<HashSet<_>>(),
                        )
                    })
                    .collect::<HashMap<_, _>>()
            });
            if s.len() == o.len() {
                let mut res = [0; 9];
                s.into_iter()
                    .map(|(n, e)| o.iter().find(|(_, f)| &&e == f).map(|(m, _)| (n, *m)))
                    .try_for_each(|o| {
                        res[o?.0 as usize - 1] = o?.1;
                        Some(())
                    })
                    .map(|_| res)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn rotation_l(&mut self) {
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

fn ergebnis_wenn_anders(f: &str, (a, b, c): (usize, usize, usize)) -> Option<String> {
    if a < b && b < c {
        None
    } else {
        Some(format!("{f}{} {} {}", a + 1, b + 1, c + 1)) // null-indexed
    }
}

fn formattiere_ergebnis(
    rotiert: bool,
    z_b: (usize, usize, usize),
    z_1: (usize, usize, usize),
    z_2: (usize, usize, usize),
    z_3: (usize, usize, usize),
    s_b: (usize, usize, usize),
    s_1: (usize, usize, usize),
    s_2: (usize, usize, usize),
    s_3: (usize, usize, usize),
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

type PermutationsIndex = (usize, usize, usize);
const PERMUTATIONS: [PermutationsIndex; 6] = [
    (0, 1, 2),
    (0, 2, 1),
    (1, 0, 2),
    (1, 2, 0),
    (2, 0, 1),
    (2, 1, 0),
];
type PermutationsIndex8 = (
    PermutationsIndex,
    PermutationsIndex,
    PermutationsIndex,
    PermutationsIndex,
    PermutationsIndex,
    PermutationsIndex,
    PermutationsIndex,
    PermutationsIndex,
);
fn get_moeglichkeiten() -> impl ParallelIterator<Item = PermutationsIndex8> {
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
        .map(|(((((((a, b), c), d), e), f), g), h)| (a, b, c, d, e, f, g, h))
}
pub fn a3(sudokus: String) {
    let (original, neu) = sudokus
        .replace('\r', "") // einfacher um die beiden Sudokus zu trennen (nur auf ein doppeltes `\n` trennen)
        .split("\n\n")
        .map(Sudoku::from)
        .collect_tuple()
        .expect("Falsches Format!");
    let neu_rotiert = {
        let mut a = neu; // neu wird kopiert
        a.rotation_l();
        a
    };
    println!(
        "{}",
        get_moeglichkeiten()
            .find_map_any(|(z_b, z_1, z_2, z_3, s_b, s_1, s_2, s_3)| {
                let mut new = original; // original wird kopiert
                tauschen(&mut new, z_b, 0, Sudoku::vertausche_zeilen_block);
                tauschen(&mut new, z_1, 0, Sudoku::vertausche_zeile);
                tauschen(&mut new, z_2, 3, Sudoku::vertausche_zeile);
                tauschen(&mut new, z_3, 6, Sudoku::vertausche_zeile);
                tauschen(&mut new, s_b, 0, Sudoku::vertausche_spalten_block);
                tauschen(&mut new, s_1, 0, Sudoku::vertausche_spalte);
                tauschen(&mut new, s_2, 3, Sudoku::vertausche_spalte);
                tauschen(&mut new, s_3, 6, Sudoku::vertausche_spalte);
                let wie_neu = new.aehnlich(&neu);
                let wie_neu_rotiert = new.aehnlich(&neu_rotiert);
                wie_neu.or(wie_neu_rotiert).map(|umbenennung| {
                    formattiere_ergebnis(
                        wie_neu_rotiert.is_some(),
                        z_b,
                        z_1,
                        z_2,
                        z_3,
                        s_b,
                        s_1,
                        s_2,
                        s_3,
                        umbenennung,
                    )
                })
            })
            .unwrap_or_else(|| "Unterschiedliche Sudokus!".to_string())
    );
}

fn tauschen(
    s: &mut Sudoku,
    p: (usize, usize, usize),
    o: usize,
    mut f: impl FnMut(&mut Sudoku, usize, usize),
) {
    match p {
        (1, 2, 0) => {
            f(s, o, o + 1);
            f(s, o, o + 2);
        }
        (2, 0, 1) => {
            f(s, o, o + 1);
            f(s, o + 1, o + 2);
        }
        (0, 2, 1) => f(s, o + 1, o + 2),
        (1, 0, 2) => f(s, o, o + 1),
        (2, 1, 0) => f(s, o, o + 2),
        (0, 1, 2) => {}
        _ => unreachable!(),
    }
}
