use aufgaben_helfer::loese_aufgabe;
use itertools::Itertools;
use tinyvec::ArrayVec;

use std::{collections::HashMap, fmt::Debug};

const MAX_STAPEL_SIZE: usize = 16;

#[derive(PartialEq, Eq, Clone, Debug, Hash, Default, Copy)]
struct Stapel {
    stapel: ArrayVec<[u8; MAX_STAPEL_SIZE]>,
}
impl From<&str> for Stapel {
    fn from(s: &str) -> Self {
        let mut zeilen = s
            .trim_start_matches('\u{feff}') // manchmal gibt es ein BOM am Anfang eines Strings
            .split_ascii_whitespace();
        let groesse: usize = zeilen
            .next()
            .unwrap()
            .parse()
            .expect("Gesamtgröße ist keine Zahl");
        let mut stapel = zeilen
            .map(|groesse| groesse.parse().expect("Größe ist keine Zahl"))
            .collect::<ArrayVec<_>>();
        stapel.reverse();
        assert_eq!(groesse, stapel.len());
        Self { stapel }
    }
}
impl Stapel {
    // TODO inefficient
    fn wenden_und_essen(&self, index: usize, normalisieren: bool) -> Self {
        assert!(!self.stapel.is_empty());
        assert!(index < self.stapel.len());
        let mut neuer_stapel = ArrayVec::new();
        let gegessen = self.stapel[index];
        for i in 0..index {
            let mut tmp = self.stapel[i];
            if normalisieren && tmp > gegessen {
                tmp -= 1;
            }
            neuer_stapel.push(tmp);
        }
        for i in 0..self.stapel.len() - index - 1 {
            let mut tmp = self.stapel[self.stapel.len() - 1 - i];
            if normalisieren && tmp > gegessen {
                tmp -= 1;
            }
            neuer_stapel.push(tmp);
        }
        Self {
            stapel: neuer_stapel,
        }
    }

    fn is_sorted(&self) -> bool {
        let mut letztes = u8::MAX;
        for pancake in self.stapel {
            if pancake > letztes {
                return false;
            }
            letztes = pancake;
        }
        true
    }

    fn print(&self, max: usize) {
        for pancake in self.stapel.iter().rev() {
            let groesse = *pancake as usize;
            print!("({:length$}) ", groesse, length = max.to_string().len());
            print!("{}", " ".repeat(max - groesse));
            print!("{}", "_".repeat(groesse * 2));
            println!("{}", " ".repeat(max - groesse));
        }
    }
}
#[derive(PartialEq, Eq, Clone, Debug, Hash, Default, Copy)]
struct Status {
    stapel: ArrayVec<[Stapel; MAX_STAPEL_SIZE]>,
}

fn stapel_durchprobieren(
    gesehen: &mut HashMap<Stapel, ArrayVec<[u8; MAX_STAPEL_SIZE]>>,
    stapel: Stapel,
) -> ArrayVec<[u8; MAX_STAPEL_SIZE]> {
    if let Some(status) = gesehen.get(&stapel) {
        *status
    } else {
        let best = if stapel.is_sorted() {
            ArrayVec::new()
        } else {
            let mut best: Option<ArrayVec<[u8; MAX_STAPEL_SIZE]>> = None;
            // test all states & pick the best one
            for i in 0..stapel.stapel.len() {
                let neuer_stapel = stapel.wenden_und_essen(i, false);
                let neuer_status = stapel_durchprobieren(gesehen, neuer_stapel);
                if best
                    .map(|b| b.len())
                    .filter(|l| *l <= neuer_status.len() + 1)
                    .is_none()
                {
                    best = Some(neuer_status);
                    best.as_mut().unwrap().push(i as u8);
                    println!();
                    neuer_stapel.print(7);
                    println!();
                    println!("{best:?}");
                    println!();
                }
            }
            best.unwrap()
        };
        gesehen.insert(stapel, best);
        best
    }
}

fn print(mut stapel: Stapel, wende_und_ess_operationen: &ArrayVec<[u8; MAX_STAPEL_SIZE]>) {
    println!("A(s) = {}", wende_und_ess_operationen.len());
    println!("Anfangsstapel:");
    let anfangs_groesse = stapel.stapel.len();
    stapel.print(anfangs_groesse);
    println!();

    for wende_und_ess_operation in wende_und_ess_operationen {
        println!("Ess-und-Wende-Operation bei: {wende_und_ess_operation}");
        println!();
        stapel = stapel.wenden_und_essen(*wende_und_ess_operation as _, false);
        stapel.print(anfangs_groesse);
        println!();
    }
}

pub fn a3(eingabe: String) {
    let anfangs_stapel = Stapel::from(eingabe.as_str());
    let mut gesehen = HashMap::new();
    print(
        anfangs_stapel,
        &stapel_durchprobieren(&mut gesehen, anfangs_stapel),
    );
}

fn main() {
    loese_aufgabe(a3);
}

#[cfg(test)]
mod tests {
    use tinyvec::ArrayVec;

    use crate::Stapel;

    #[test]
    fn parse() {
        let cut = "4\n1\n3\n4\n2\n".into();
        assert_eq!(
            Stapel {
                stapel: ArrayVec::from_iter([1, 3, 4, 2]),
            },
            cut
        );
    }

    #[test]
    fn wenden_und_essen() {
        let cut = Stapel::from("4\n1\n2\n3\n4");
        assert_eq!(cut.wenden_und_essen(2, true), "3\n1\n2\n3".into());
        assert_eq!(cut.wenden_und_essen(2, false), "3\n1\n3\n4".into());
    }
}
