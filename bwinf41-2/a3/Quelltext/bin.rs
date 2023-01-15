use itertools::Itertools;
use tinyvec::ArrayVec;

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc},
    time::Instant,
};

const MAX_STAPEL_GROESSE: usize = 16;

#[derive(PartialEq, Eq, Clone, Debug, Hash, Default, Copy)]
struct Stapel {
    stapel: ArrayVec<[u8; MAX_STAPEL_GROESSE]>,
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
    fn wenden_und_essen(&self, index: u8, normalisieren: bool) -> Self {
        assert!(self.stapel.len() > index as usize);
        let mut neuer_stapel = ArrayVec::new();
        let gegessen = self.stapel[index as usize];
        for i in 0..index {
            let mut tmp = self.stapel[i as usize];
            if normalisieren && tmp > gegessen {
                tmp -= 1;
            }
            neuer_stapel.push(tmp);
        }
        for i in 0..self.stapel.len() - index as usize - 1 {
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

// fn stapel_durchprobieren(
//     gesehen: &mut HashMap<Stapel, ArrayVec<[u8; MAX_STAPEL_GROESSE]>>,
//     stapel: Stapel,
// ) -> ArrayVec<[u8; MAX_STAPEL_GROESSE]> {
//     if let Some(status) = gesehen.get(&stapel) {
//         *status
//     } else {
//         let beste_operationen = if stapel.is_sorted() {
//             ArrayVec::new()
//         } else {
//             let mut beste_operationen: Option<ArrayVec<[u8; MAX_STAPEL_GROESSE]>> = None;
//             // test all states & pick the best one
//             for i in 0..stapel.stapel.len() {
//                 let neuer_stapel = stapel.wenden_und_essen(i, true);
//                 let mut neuer_status = stapel_durchprobieren(gesehen, neuer_stapel);
//                 if beste_operationen
//                     .map(|b| b.len())
//                     .filter(|l| *l <= neuer_status.len() + 1)
//                     .is_none()
//                 {
//                     neuer_status.push(i as u8);
//                     beste_operationen = Some(neuer_status);
//                 }
//             }
//             beste_operationen.unwrap()
//         };
//         gesehen.insert(stapel, beste_operationen);
//         beste_operationen
//     }
// }

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct Flip(u8);
impl Flip {
    fn new(v: Option<u8>) -> Self {
        Self(v.unwrap_or(u8::MAX))
    }

    fn as_option(&self) -> Option<u8> {
        (self.0 < u8::MAX).then_some(self.0)
    }
}
impl From<Option<u8>> for Flip {
    fn from(value: Option<u8>) -> Self {
        Self::new(value)
    }
}
impl From<Flip> for Option<u8> {
    fn from(value: Flip) -> Self {
        value.as_option()
    }
}

fn stapel_durchprobieren2(
    gesehen: &HashMap<Stapel, (usize, Flip)>,
    stapel: Stapel,
) -> (usize, Flip) {
    if let Some(status) = gesehen.get(&stapel) {
        *status
    } else {
        let beste_operationen = if stapel.is_sorted() {
            (0, None.into())
        } else {
            let mut beste_operationen: Option<(usize, Flip)> = None;
            // test all states & pick the best one
            for i in 0..stapel.stapel.len() as u8 {
                let neuer_stapel = stapel.wenden_und_essen(i, true);
                let (anzahl, _) = stapel_durchprobieren2(gesehen, neuer_stapel);
                if beste_operationen.filter(|(l, _)| l <= &anzahl).is_none() {
                    beste_operationen = Some((anzahl + 1, Some(i).into()));
                }
            }
            beste_operationen.unwrap()
        };
        beste_operationen
    }
}

fn print(mut stapel: Stapel, gesehen: &HashMap<Stapel, (usize, Flip)>) {
    println!("Anfangsstapel:");
    let anfangs_groesse = stapel.stapel.len();
    stapel.print(anfangs_groesse);
    println!();

    let mut stapel_to_print = stapel;

    while let Some(wende_und_ess_operation) = gesehen.get(&stapel).and_then(|(_, f)| f.as_option())
    {
        println!("Ess-und-Wende-Operation bei: {wende_und_ess_operation}");
        println!();
        stapel = stapel.wenden_und_essen(wende_und_ess_operation, true);
        stapel_to_print = stapel_to_print.wenden_und_essen(wende_und_ess_operation, false);
        stapel_to_print.print(anfangs_groesse);
        println!();
    }
}

// pub fn a3_a(eingabe: String) {
//     let anfangs_stapel = Stapel::from(eingabe.as_str());
//     let mut gesehen = HashMap::new();
//     print(
//         anfangs_stapel,
//         &stapel_durchprobieren(&mut gesehen, anfangs_stapel),
//     );
// }

// copied from unstable u64::div_ceil
#[inline]
pub const fn div_ceil(a: u64, b: u64) -> u64 {
    let d = a / b;
    let r = a % b;
    if r > 0 && b > 0 {
        d + 1
    } else {
        d
    }
}

macro_rules! exec_time {
    () => {
        $crate::eprintln!("[{}:{}]", $crate::file!(), $crate::line!())
    };
    ($val:expr $(,)?) => {{
        let start = std::time::Instant::now();
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                eprintln!("[{}:{}] {} = {:?}",
                    file!(), line!(), stringify!($val), start.elapsed());
                tmp
            }
        }
    }};
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

pub fn a3_b(limit: u8) {
    let start = Instant::now();

    let mut gesehen = HashMap::new();
    let mut worst_cases = Vec::new();
    for i in 1..=limit {
        let mut worst_case: Option<(Stapel, usize, Flip)> = None;
        let factorial = (1..=i as u64).product1().unwrap_or(1_u64);
        let thread_count = std::thread::available_parallelism().unwrap().get();
        let chunk_size = div_ceil(factorial, thread_count as u64) as usize;
        let permutations = (1..=i)
            .permutations(i as usize)
            .chunks(chunk_size)
            .into_iter()
            .map(|c| c.collect_vec())
            .collect_vec();
        let mut gesehen_neu = HashMap::new();
        let gesehen_arc = Arc::new(gesehen);
        for handle in permutations.into_iter().map(|chunk| {
            let gesehen_clone = gesehen_arc.clone();
            std::thread::spawn(move || {
                let gesehen = gesehen_clone;
                let mut worst_case: Option<(Stapel, usize, Flip)> = None;
                let mut gesehen_neu = HashMap::new();
                for s in chunk {
                    let stapel = Stapel {
                        stapel: ArrayVec::from_iter(s),
                    };
                    let (laenge, index) = stapel_durchprobieren2(&gesehen, stapel);
                    gesehen_neu.insert(stapel, (laenge, index));
                    if worst_case.filter(|(_, l, _)| l >= &laenge).is_none() {
                        worst_case = Some((stapel, laenge, index));
                    }
                }
                (worst_case, gesehen_neu)
            })
        }) {
            let (wc, gesehen_n) = handle.join().unwrap();
            gesehen_neu.extend(gesehen_n);
            if let Some((stapel, laenge, index)) = wc {
                if worst_case.filter(|(_, l, _)| l >= &laenge).is_none() {
                    worst_case = Some((stapel, laenge, index));
                }
            }
        }
        worst_cases.push(worst_case);
        gesehen = Arc::try_unwrap(gesehen_arc).unwrap();
        gesehen.extend(gesehen_neu);

        if let Some((stapel, laenge, index)) = worst_case {
            println!("P({i}) = {laenge}");
            println!();
            println!("Beispiel:");
            print(stapel, &gesehen);
            println!();
        }
    }

    println!("{:^5} | {:^5}", "n", "P(n)");
    println!("{:-^5}-+-{:-^5}", "", "");
    for (n, pn) in worst_cases
        .into_iter()
        .enumerate()
        .filter_map(|(n, w)| w.map(|(_, l, _)| (n + 1, l)))
    {
        println!("{n:^5} | {pn:^5}");
    }

    println!();
    println!("Ausführungsdauer: {}ms", start.elapsed().as_millis());
}

fn main() {
    match std::env::args().nth(1).and_then(|n| n.parse::<u8>().ok()) {
        Some(limit) if std::env::args().count() == 2 => a3_b(limit),
        _ => todo!(), //loese_aufgabe(a3_a),
    }
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
