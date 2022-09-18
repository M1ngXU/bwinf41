use std::{str::FromStr, fmt::Debug};

use itertools::Itertools;

#[derive(PartialEq, Eq, Clone, Copy, Default)]
struct Sudoku {
    feld: [[Option<u8>; 9]; 9],
}
impl Debug for Sudoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for zeile in self.feld {
			writeln!(f, "{}", zeile.iter().map(|n| n.unwrap_or_default()).join(" "))?;
		}
		Ok(())
    }
}
impl FromStr for Sudoku {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            feld: s
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
        self.feld.swap(a, b);
    }

    fn vertausche_spalte(&mut self, a: usize, b: usize) {
        self.feld.iter_mut().for_each(|zeile| zeile.swap(a, b));
    }

    fn vertausche_zeilen_block(&mut self, a: usize, b: usize) {
        for i in 0..3 {
            self.vertausche_zeile(3 * a + i, 3 * b + i);
        }
    }

    fn vertausche_spalten_block(&mut self, a: usize, b: usize) {
        self.feld.iter_mut().for_each(|zeile| {
            zeile.swap(3 * a, 3 * b);
            zeile.swap(3 * a + 1, 3 * b);
            zeile.swap(3 * a, 3 * b);
        });
    }
}

pub fn a3(sudokus: String) {}

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
		assert_eq!("0 0 0 0 0 0 0 0 1
0 0 4 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0 0
3 0 0 0 0 0 0 0 2
".parse::<Sudoku>().unwrap(), cut);
    }
}