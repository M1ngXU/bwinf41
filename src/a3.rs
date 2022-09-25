use std::{collections::HashSet, fmt::Debug, str::FromStr};

use itertools::Itertools;
use rayon::prelude::*;

#[derive(PartialEq, Eq, Clone, Copy, Default)]
struct Sudoku {
    feld: [[Option<u8>; 9]; 9],
}
impl Debug for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for zeile in self.feld {
            writeln!(
                f,
                "{}",
                zeile.iter().map(|n| n.unwrap_or_default()).join(" ")
            )?;
        }
        Ok(())
    }
}
impl FromStr for Sudoku {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            feld: s
                .trim_start_matches("\u{feff}")
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
                .unwrap(),
        })
    }
}
impl Sudoku {
    fn aehnlich(&self, other: &Self) -> bool {
        self.feld.iter().flatten().map(Option::is_some).eq(other
            .feld
            .iter()
            .flatten()
            .map(Option::is_some))
            && {
                let s = self
                    .feld
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
                    .map(|(_n, group_iter)| {
                        group_iter.map(|(x, y, _n)| (x, y)).collect::<HashSet<_>>()
                    })
                    .collect_vec();
                let o = other
                    .feld
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
                    .map(|(_n, group_iter)| {
                        group_iter.map(|(x, y, _n)| (x, y)).collect::<HashSet<_>>()
                    })
                    .collect_vec();
                s.len() == o.len() && s.into_iter().all(|e| o.iter().find(|f| &&e == f).is_some())
            }
    }

    fn rotation_r(&mut self) {
        // = transpose + vertikal spiegeln
        let feld = self.feld.clone();
        for y in 0..9 {
            for x in 0..9 {
                self.feld[y][x] = feld[8 - x][y];
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

pub fn a3(sudokus: String) {
    let (orig, neu) = sudokus
        .replace("\r", "")
        .split("\n\n")
        .filter_map(|s| s.parse::<Sudoku>().ok())
        .collect_tuple()
        .expect("Falsches Format!");
    let orig_rotiert = {
        let mut a = orig.clone();
        a.rotation_r();
        a
    };
    let permutations = [
        (0, 1, 2),
        (0, 2, 1),
        (1, 0, 2),
        (1, 2, 0),
        (2, 0, 1),
        (2, 1, 0),
    ];
    println!(
        "{:?}",
        permutations
            .iter()
            .cartesian_product(permutations)
            .cartesian_product(permutations)
            .cartesian_product(permutations)
            .cartesian_product(permutations)
            .cartesian_product(permutations)
            .cartesian_product(permutations)
            .cartesian_product(permutations)
            .par_bridge()
            .into_par_iter()
            //.inspect(|v| println!("{v:?}"))
            .find_any(|(((((((z_b, z_1), z_2), z_3), s_b), s_1), s_2), s_3)| {
                let mut new = neu.clone();
                tauschen(&mut new, z_b, 0, Sudoku::vertausche_zeilen_block);
                tauschen(&mut new, z_1, 0, Sudoku::vertausche_zeile);
                tauschen(&mut new, z_2, 3, Sudoku::vertausche_zeile);
                tauschen(&mut new, z_3, 6, Sudoku::vertausche_zeile);
                tauschen(&mut new, s_b, 0, Sudoku::vertausche_spalten_block);
                tauschen(&mut new, s_1, 0, Sudoku::vertausche_spalte);
                tauschen(&mut new, s_2, 3, Sudoku::vertausche_spalte);
                tauschen(&mut new, s_3, 6, Sudoku::vertausche_spalte);
                new.aehnlich(&orig) || new.aehnlich(&orig_rotiert)
            })
    );
}

fn tauschen(s: &mut Sudoku, p: &(usize, usize, usize), o: usize, mut f: impl FnMut(&mut Sudoku, usize, usize)) {
    match p {
        &(1, 2, 0) => {f(s, o, o + 1);f(s, o, o + 2);},
        &(2, 0, 1) => {f(s, o, o + 1);f(s, o + 1, o + 2);},
        &(0, 2, 1) => f(s, o + 1, o + 2),
        &(1, 0, 2) => f(s, o, o + 1),
        &(2, 1, 0) => f(s, o, o + 2),
        &(0, 1, 2) => {},
        _ => unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::Sudoku;

    #[test]
    fn rotate() {
        let mut cut: Sudoku = "1 0 0 0 0 0 0 0 2
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 4 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 3
"
        .parse()
        .unwrap();
        cut.rotation_r();
        assert_eq!(
            "0 0 0 0 0 0 0 0 1
0 0 4 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
3 0 0 0 0 0 0 0 2
"
            .parse::<Sudoku>()
            .unwrap(),
            cut
        );
    }

    #[test]
    fn aehnlich() {
        let cut: Sudoku = "1 0 0 0 0 0 0 1 2
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 4 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 3
"
        .parse()
        .unwrap();
        assert!(cut.aehnlich(
            &"2 0 0 0 0 0 0 2 3
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 5 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 4
"
            .parse::<Sudoku>()
            .unwrap()
        ));
        assert!(cut.aehnlich(
            &"4 0 0 0 0 0 0 4 3
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 5 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 2
"
            .parse::<Sudoku>()
            .unwrap()
        ));
        assert!(!cut.aehnlich(
            &"0 4 0 0 0 0 0 2 3
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 5 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 2
"
            .parse::<Sudoku>()
            .unwrap()
        ));
        assert!(!cut.aehnlich(
            &"4 0 0 0 0 0 0 3 3
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 5 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 2
"
            .parse::<Sudoku>()
            .unwrap()
        ));
    }
}
