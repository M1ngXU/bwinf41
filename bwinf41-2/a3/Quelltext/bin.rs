use cudarc::{
    nvrtc::Ptx,
    prelude::{CudaDeviceBuilder, CudaSlice, LaunchAsync, LaunchConfig, ValidAsZeroBits},
};
use itertools::Itertools;
use tinyvec::ArrayVec;

use std::{collections::HashMap, fmt::{Debug, Display}, sync::Arc, time::Instant};

const MAX_STAPEL_GROESSE: usize = 16;

type Array = ArrayVec<[u8; MAX_STAPEL_GROESSE]>;

#[derive(PartialEq, Eq, Clone, Debug, Hash, Default)]
struct Stapel {
    stapel: Array,
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
            .collect::<Array>();
        stapel.reverse();
        assert_eq!(groesse, stapel.len());
        Self { stapel }
    }
}
impl Stapel {
    // TODO inefficient
    fn wenden_und_essen(&self, index: u8, normalisieren: bool) -> Self {
        assert!(self.stapel.len() > index as usize);
        let mut neuer_stapel = Array::new();
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
        for pancake in &self.stapel {
            if pancake > &letztes {
                return false;
            }
            letztes = *pancake;
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
//     gesehen: &mut HashMap<Stapel, Array,
//     stapel: Stapel,
// ) -> Array {
//     if let Some(status) = gesehen.get(&stapel) {
//         *status
//     } else {
//         let beste_operationen = if stapel.is_sorted() {
//             Array::new()
//         } else {
//             let mut beste_operationen: Option<Array> = None;
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

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(transparent)]
struct Flip(u8);
unsafe impl ValidAsZeroBits for Flip {}
impl Flip {
    fn new(v: Option<u8>) -> Self {
        Self(v.unwrap_or(u8::MAX))
    }

    fn as_option(&self) -> Option<u8> {
        (self.0 < u8::MAX).then_some(self.0)
    }
}
impl Display for Flip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_option().map_or("None".to_string(), |n| n.to_string()))
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

fn stapel_durchprobieren2(gesehen: &HashMap<Stapel, (u8, Flip)>, stapel: &Stapel) -> (u8, Flip) {
    if let Some(status) = gesehen.get(&stapel) {
        *status
    } else if stapel.is_sorted() {
        (0, None.into())
    } else {
        let mut beste_operationen: Option<(u8, Flip)> = None;
        // test all states & pick the best one
        for i in 0..stapel.stapel.len() as u8 {
            let neuer_stapel = stapel.wenden_und_essen(i, true);
            let (anzahl, _) = stapel_durchprobieren2(gesehen, &neuer_stapel);
            if beste_operationen.filter(|(l, _)| l <= &anzahl).is_none() {
                beste_operationen = Some((anzahl + 1, Some(i).into()));
            }
        }
        beste_operationen.unwrap()
    }
}

fn print(mut stapel: Stapel, gesehen: &HashMap<Stapel, (u8, Flip)>) {
    println!("Anfangsstapel:");
    let anfangs_groesse = stapel.stapel.len();
    stapel.print(anfangs_groesse);
    println!();

    let mut stapel_to_print = stapel.clone();

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

fn print2(mut stapel: Stapel, gesehen: &HashMap<u64, (u8, Flip)>) {
    println!("Anfangsstapel:");
    let anfangs_groesse = stapel.stapel.len();
    stapel.print(anfangs_groesse);
    println!();

    let mut stapel_to_print = stapel.clone();

    while let Some(wende_und_ess_operation) = gesehen.get(&enumerate_permutation(stapel.stapel, &mut Default::default())).and_then(|(_, f)| f.as_option())
    {
        println!("Ess-und-Wende-Operation bei: {wende_und_ess_operation}");
        stapel = stapel.wenden_und_essen(wende_und_ess_operation, true);
        stapel_to_print = stapel_to_print.wenden_und_essen(wende_und_ess_operation, false);
        stapel_to_print.print(anfangs_groesse);
        println!();
        if stapel.is_sorted() {
            break;
        }
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
pub const fn div_ceil(a: usize, b: usize) -> usize {
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

fn enumerate_permutation(mut x: Array, indeces: &mut Array) -> u64 {
    let mut result = 0;
    indeces.truncate(0);
    for i in 0..x.len() {
        let index = x.iter().position(|n| *n == i as u8 + 1).unwrap() as usize;
        indeces.push(index as u8);
        x.remove(index);
    }
    let mut fact = 1;
    for (i, index) in indeces.iter().rev().enumerate() {
        if i > 1 {
            fact *= i as u64;
        }
        result += *index as u64 * fact;
    }
    result
}

#[inline]
fn permutation_by_enumeration(mut i: u64, n: u8, indeces: &mut Array) -> Array {
    let mut fact = (1..=n as u64).product::<u64>();
    indeces.truncate(0);
    let mut result = Array::new();
    for j in 0..n {
        fact /= n as u64 - j as u64;
        indeces.push((i / fact) as u8);
        i %= fact;
    }
    for n in (0..n).rev() {
        result.insert(indeces[n as usize] as usize, n as u8 + 1);
    }
    result
}

pub fn a3_b1(limit: u8) {
    let start = Instant::now();

    let mut gesehen = HashMap::with_capacity(
        (0..limit as u64)
            .map(|i| (1..=i).product::<u64>())
            .sum::<u64>() as usize,
    );
    let mut worst_cases = Vec::new();
    for i in 1..=limit {
        let mut worst_case: Option<(Stapel, u8, Flip)> = None;
        let factorial = (1..=i as usize).product1().unwrap_or(1_usize);
        let thread_count = std::thread::available_parallelism().unwrap().get();
        let chunk_size = div_ceil(factorial, thread_count);
        let permutations = (1..=i)
            .permutations(i as usize)
            .chunks(chunk_size)
            .into_iter()
            .map(|c| c.collect_vec())
            .collect_vec();
        let mut gesehen_neu = HashMap::with_capacity(factorial);
        let gesehen_arc = Arc::new(gesehen);
        for handle in permutations.into_iter().map(|chunk| {
            let gesehen_clone = gesehen_arc.clone();
            std::thread::spawn(move || {
                // let chunk_start = chunk * chunk_size;
                // let chunk_end = ((chunk + 1) * chunk_size).min(factorial);
                let gesehen = gesehen_clone;
                let mut worst_case: Option<(Stapel, u8, Flip)> = None;
                let mut gesehen_neu = HashMap::with_capacity(chunk_size);
                // let mut tmp = Array::new();
                for s in chunk {
                    let stapel = Stapel {
                        stapel: s.into_iter().collect(), //permutation_by_enumeration(s, i as usize, &mut tmp),
                    };
                    let (laenge, index) = stapel_durchprobieren2(&gesehen, &stapel);
                    gesehen_neu.insert(stapel.clone(), (laenge, index));
                    if worst_case
                        .as_ref()
                        .filter(|(_, l, _)| l >= &laenge)
                        .is_none()
                    {
                        worst_case = Some((stapel, laenge, index));
                    }
                }
                (worst_case, gesehen_neu)
            })
        }) {
            let (wc, gesehen_n) = handle.join().unwrap();
            gesehen_neu.extend(gesehen_n);
            if let Some((stapel, laenge, index)) = wc {
                if worst_case
                    .as_ref()
                    .filter(|(_, l, _)| l >= &laenge)
                    .is_none()
                {
                    worst_case = Some((stapel, laenge, index));
                }
            }
        }
        worst_cases.push(worst_case.clone());
        gesehen = Arc::try_unwrap(gesehen_arc).unwrap();
        gesehen.extend(gesehen_neu);

        if let Some((stapel, laenge, _)) = worst_case {
            println!("P({i}) = {laenge}");
            // println!();
            // println!("Beispiel:");
            // print(stapel, &gesehen);
            // println!();
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

const THREADS: u32 = 256;
const BLOCKS: u32 = 64;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
struct Bestes(u64, u8, Flip);
unsafe impl ValidAsZeroBits for Bestes {}

fn a3_b2(limit: u8) {
    let start = Instant::now();

    let mut worst_cases = Vec::new();
    worst_cases.push(0);

    let device = CudaDeviceBuilder::new(0)
        .with_ptx(
            Ptx::Image(include_bytes!("a/kernel.ptx").to_vec().into_iter().map(|i| i as i8).collect_vec()), // r"bwinf41-2\a3\Quelltext\kernel.cu",
            "cuda",
            &["run_permutations"],
        )
        .build()
        .unwrap();
    let run_permutations = device.get_func("cuda", "run_permutations").unwrap();

    let mut prior = unsafe {device.alloc_async(1).unwrap()};
    device
        .copy_into_async(vec![(0, None.into())], &mut prior)
        .unwrap();
    let mut current: CudaSlice<(u8, Flip)>;
    let mut fact = 1;
    let mut bestes_gefundene = device
        .alloc_zeros_async((THREADS * BLOCKS) as usize)
        .unwrap();
    let mut beste_box = Box::new([Bestes(0, 0, None.into()); (THREADS * BLOCKS) as usize]);
    let bestes = beste_box.as_mut_slice();

    let mut gesehen = HashMap::new();

    for i in 2..=limit {
        fact *= i as u64;
        let threads = if fact < THREADS as u64 { 1 } else { THREADS };
        let blocks = if fact < (THREADS * BLOCKS) as u64 { 1 } else { BLOCKS };
        current = unsafe{device.alloc_async(fact as usize).unwrap()};
        unsafe {
            run_permutations.clone().launch_async(
                LaunchConfig {
                    block_dim: (threads, 1, 1),
                    grid_dim: (blocks, 1, 1),
                    shared_mem_bytes: 0,
                },
                (&prior, &mut current, &mut bestes_gefundene, i, fact),
            )
        }
        .unwrap();

        let mut gesehen_neu = vec![Default::default(); fact as usize];
        device
            .sync_copy_from(&current, gesehen_neu.as_mut_slice())
            .unwrap();
        gesehen.extend(
            gesehen_neu
                .into_iter().enumerate().map(|(i, b)| (i as u64, b))
        );
        device.sync_copy_from(&bestes_gefundene, bestes).unwrap();
        let Bestes(enumeration, laenge, flip) = bestes[..(threads * blocks) as usize]
            .iter()
            .max_by_key(|Bestes(_, laenge, _)| laenge)
            .unwrap();

        if laenge == &u8::MAX {
            println!("Nothing found for {i}");
        } else {
            worst_cases.push(*laenge);
            println!("P({i}) = {laenge}");
            // println!("Last flip at {flip}");
            // print2(Stapel {
            //     stapel: permutation_by_enumeration(
            //         *enumeration,
            //         i,
            //         &mut Default::default(),
            //     ),
            // }, &gesehen);
            println!();
        }

        prior = current;
    }

    println!("{:^5} | {:^5}", "n", "P(n)");
    println!("{:-^5}-+-{:-^5}", "", "");
    for (n, pn) in worst_cases
        .into_iter()
        .enumerate()
    {
        println!("{:^5} | {:^5}", n + 1, pn);
    }
    println!();
    println!("Ausführungsdauer: {}ms", start.elapsed().as_millis());
}

fn main() {
    // println!("{:?}", enumerate_permutation(vec![2, 0, 4, 1, 3]));
    // println!("{:?}", permutation_by_enumeration(37, 5));
    // panic!();
    match std::env::args().nth(1).and_then(|n| n.parse::<u8>().ok()) {
        Some(limit) if std::env::args().count() == 2 => a3_b2(limit),
        _ => todo!(), //loese_aufgabe(a3_a),
    }
}

#[cfg(test)]
mod tests {
    use crate::{Array, Stapel};

    #[test]
    fn parse() {
        let cut = "4\n1\n3\n4\n2\n".into();
        assert_eq!(
            Stapel {
                stapel: Array::from_iter([1, 3, 4, 2]),
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
