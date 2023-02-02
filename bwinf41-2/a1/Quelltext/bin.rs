use image::{ImageBuffer, Rgb};
use rand::{
    distributions::Uniform,
    prelude::Distribution,
    seq::{IteratorRandom, SliceRandom},
    thread_rng, Rng,
};
use rayon::{
    prelude::{
        IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelBridge,
        ParallelIterator,
    },
    slice::ParallelSliceMut,
};
use show_image::create_window;
use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    fmt::{Display, Formatter},
    hash::Hash,
    ops::Deref,
    sync::Arc,
    time::Instant,
};

use aufgaben_helfer::loese_aufgabe;
use imageproc::drawing::{draw_hollow_circle_mut, draw_line_segment_mut};
use itertools::Itertools;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
struct Ort {
    x: i64,
    y: i64,
    s: u128,
}
impl Display for Ort {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{:0<6}|{}.{:0<6}",
            self.x / 1_000_000,
            self.x % 1_000_000,
            self.y / 1_000_000,
            self.y % 1_000_000
        )
    }
}
impl Ort {
    fn moegliche_abbiegung(&self, von: &Self, zu: &Self) -> bool {
        let a = (self.x - von.x, self.y - von.y);
        let b = (zu.x - self.x, zu.y - self.y);
        let skalarprodukt = a.0 * b.0 + a.1 * b.1;
        skalarprodukt > 0
    }

    fn kosten_zu(&self, zu: &Self) -> u64 {
        ((self.x - zu.x).pow(2) + (self.y - zu.y).pow(2)) as u64
    }
}

#[derive(Default, Clone, Hash, Debug, PartialEq, Eq)]
struct OrtNachKosten {
    besucht: u128,
    current: Kante,
}
#[derive(Clone, Hash, Debug, Default)]
struct Wrapper(OrtNachKosten, u64, Vec<Kante>);
impl Deref for Wrapper {
    type Target = OrtNachKosten;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl PartialEq for Wrapper {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}
impl Eq for Wrapper {}
impl PartialOrd for Wrapper {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Wrapper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.1.cmp(&other.1).reverse()
    }
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, Default)]
struct Kante {
    von: Ort,
    zu: Ort,
}

fn best_pfad(orte: Vec<Ort>, kanten: HashMap<Kante, Vec<Kante>>) -> Wrapper {
    let leer = u128::MAX ^ (1 << orte.len() - 1);
    kanten
        .clone()
        .into_par_iter()
        .filter_map(|(start, _)| {
            let mut warteschlange = BinaryHeap::<Wrapper>::new();
            let mut gesehen = HashSet::<OrtNachKosten>::new();
            let mut besucht = leer;
            besucht |= 1 << start.zu.s;
            warteschlange.push(Wrapper(
                OrtNachKosten {
                    besucht,
                    current: start,
                },
                start.von.kosten_zu(&start.zu),
                vec![start],
            ));
            while let Some(naechster) = warteschlange.pop() {
                if gesehen.contains(&naechster) {
                    continue;
                } else {
                    gesehen.insert(naechster.clone().0);
                }
                if naechster.besucht == u128::MAX {
                    assert_eq!(naechster.2[0].von, naechster.2[orte.len() - 1].zu);
                    return Some(naechster);
                }
                for naechster_ort in
                    kanten
                        .get(&naechster.current)
                        .into_iter()
                        .flatten()
                        .filter(|k| {
                            (naechster.besucht & k.zu.s) == 0
                                && (k.zu != start.von || naechster.besucht.count_zeros() == 1)
                        })
                {
                    let mut pfad = naechster.2.clone();
                    pfad.push(*naechster_ort);
                    warteschlange.push(Wrapper(
                        OrtNachKosten {
                            current: *naechster_ort,
                            besucht: besucht | naechster_ort.zu.s,
                        },
                        naechster.1 + naechster.current.zu.kosten_zu(&naechster_ort.von),
                        pfad,
                    ))
                }
            }
            None
        })
        .max_by_key(|n| n.1)
        .unwrap()
}

fn display(orte: Vec<Ort>, bester_pfad: Wrapper) {
    println!("Kosten: {}", bester_pfad.1);

    let scale = 500_000;
    let min_x = orte.iter().map(|o| o.x).min().unwrap_or_default() / scale;
    let min_y = orte.iter().map(|o| o.y).min().unwrap_or_default() / scale;
    let max_x = orte.iter().map(|o| o.x).max().unwrap_or_default() / scale;
    let max_y = orte.iter().map(|o| o.y).max().unwrap_or_default() / scale;
    let offset_x = -min_x + 100;
    let offset_y = -min_y + 100;
    let width = (max_x + offset_x + 100) as u32;
    let height = (max_y + offset_y + 100) as u32;

    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    for pixel in img.pixels_mut() {
        (*pixel).0 = [255, 255, 255];
    }

    for kante in &bester_pfad.2 {
        draw_line_segment_mut(
            &mut img,
            (
                (kante.von.x) as f32 / scale as f32 + offset_x as f32,
                (kante.von.y) as f32 / scale as f32 + offset_y as f32,
            ),
            (
                (kante.zu.x) as f32 / scale as f32 + offset_x as f32,
                (kante.zu.y) as f32 / scale as f32 + offset_y as f32,
            ),
            Rgb([0, 0, 0]),
        );
        draw_hollow_circle_mut(
            &mut img,
            (
                (kante.von.x / scale + offset_x) as i32,
                (kante.von.y / scale + offset_y) as i32,
            ),
            2,
            Rgb([0, 0, 255]),
        );
        draw_hollow_circle_mut(
            &mut img,
            (
                (kante.zu.x / scale + offset_x) as i32,
                (kante.zu.y / scale + offset_y) as i32,
            ),
            2,
            Rgb([0, 0, 255]),
        );
    }
    draw_hollow_circle_mut(
        &mut img,
        (
            (bester_pfad.2[0].von.x / scale + offset_x) as i32,
            (bester_pfad.2[0].von.y / scale + offset_y) as i32,
        ),
        5,
        Rgb([255, 0, 0]),
    );
    let window = create_window("image", Default::default()).unwrap();
    window.set_image("image", img).unwrap();
    window.wait_until_destroyed().unwrap();
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
struct KantenIndex {
    von: u64,
    zu: u64,
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Hash)]
struct Pfad {
    anfang: KantenIndex,
    ende: KantenIndex,
    anfang_mitte: Option<Arc<Pfad>>,
    ende_mitte: Option<Arc<Pfad>>,
    besucht: u64,
}

const TRIES: usize = 1_000;
fn random_pfad(orte: Vec<Ort>, kanten: HashMap<Kante, Vec<Kante>>) -> Option<Wrapper> {
    let kanten_vec = kanten.clone().into_iter().collect_vec();
    let leer = u128::MAX ^ ((1 << orte.len()) - 1);
    (0..TRIES).into_par_iter().find_map_any(|_| {
        kanten_vec
            .par_iter()
            .find_map_any(|(start, starting_nexts)| {
                let mut rng = thread_rng();
                let mut possible_nexts = starting_nexts;
                let mut result = Wrapper(
                    OrtNachKosten {
                        besucht: leer,
                        current: *start,
                    },
                    start.von.kosten_zu(&start.zu),
                    vec![*start],
                );
                result.0.besucht |= start.zu.s;
                // check that everything is visited
                while result.0.besucht != u128::MAX {
                    result.0.current = **possible_nexts
                        .iter()
                        .filter(|n| {
                            ((result.0.besucht & n.zu.s) == 0)
                                && (start.von != n.zu || result.0.besucht.count_zeros() == 1)
                        })
                        .collect_vec()
                        .choose(&mut rng)?;
                    result.0.besucht |= result.0.current.zu.s;
                    result.1 += result.0.current.von.kosten_zu(&result.0.current.zu);
                    result.2.push(result.0.current);
                    possible_nexts = kanten.get(&result.0.current)?;
                }
                Some(result)
            })
    })
}

fn get_fehler(pfad: &Vec<Ort>, kanten: &HashMap<Kante, u128>, a: usize, b: usize, max_distance: u64) -> u64 {
    (0..pfad.len() + 1)
        .map(|i| {
            let i = i % pfad.len();
            pfad[if i == a {
                b
            } else if i == b {
                a
            } else {
                i
            }]
        })
        .tuple_windows()
        .map(|(von, uber, zu)| {
            (von.kosten_zu(&uber), kanten
                .get(&Kante {
                    von: von,
                    zu: uber,
                })
                .filter(|kanten| (**kanten & zu.s) != 0)
                .is_none())
        })
        .fold(0, |a, (k, m)| a + k + if m {max_distance} else {0})
}

fn pfad_sortieren(orte: Vec<Ort>, kanten: HashMap<Kante, Vec<Kante>>) -> Option<Wrapper> {
    let max_distance = orte.iter().cartesian_product(&orte).map(|(a, b)| a.kosten_zu(b)).max().unwrap();
    let mut pfad = orte.clone();
    let mut rng = thread_rng();
    let kanten = kanten
        .into_iter()
        .map(|(von, zu)| {
            (
                von,
                zu.into_iter()
                    .map(|k| k.zu.s)
                    .reduce(|a, b| a | b)
                    .unwrap_or(0),
            )
        })
        .collect::<HashMap<Kante, u128>>();
    pfad.shuffle(&mut rng);
    let uniform = Uniform::new(0.0f32, 1.0);
    let dalpha = 0.0001;
    let mut alpha = dalpha;
    let indeces = (0..pfad.len())
        .cartesian_product(0..pfad.len())
        .collect_vec();
    let mut fehler = indeces.iter().map(|(a, b)| ((*a, *b), 0)).collect_vec();
    let mut lookup = HashSet::from([pfad.clone()]);
    let mut start;
    let mut elapsed = 0;
    let mut counter = 0u128;
    let mut clashes = 0;
    let mut letzte_verbesserung = 0;
    let mut letztes_bestes = u64::MAX;
    loop {
        start = Instant::now();
        indeces
            .par_iter()
            .map(|(a, b)| ((*a, *b), get_fehler(&pfad, &kanten, *a, *b, max_distance)))
            .collect_into_vec(&mut fehler);
        fehler.sort_unstable_by_key(|(_, w)| *w);
        elapsed += start.elapsed().as_nanos();
        let rand = (1.0 - uniform.sample(&mut rng)).ln() / -alpha;
        let i = rand.floor() as usize % fehler.len();
        let mut j = i;
        let ((mut a, mut b), mut w) = fehler[i];
        pfad.swap(a, b);
        while !lookup.insert(pfad.clone()) {
            pfad.swap(a, b);
            ((a, b), w) = fehler[j];
            j+=1;
            j%=fehler.len();
            if j==i{return None;}
            pfad.swap(a, b);
            clashes += 1;
        }
        if letztes_bestes == w {
            letzte_verbesserung += 1;
            if letzte_verbesserung == 500 {
                alpha /= 10.0;
                letzte_verbesserung = 0;
                println!("Pushdown");
            }
        } else {
            letzte_verbesserung = 0;
        }
        letztes_bestes = w / max_distance;
        counter += 1;
        if counter % 10 == 0 {
            println!("{alpha:.3}|{}|{clashes}clashes|{}ns", w / max_distance, elapsed / 10);
            elapsed = 0;
        }
        alpha += dalpha;
        if w == 0 {
            break;
        }
    }
    let pfad = pfad
        .into_iter()
        .circular_tuple_windows()
        .take(orte.len() + 1)
        .map(|(von, zu)| Kante { von, zu })
        .collect_vec();
    Some(Wrapper(
        OrtNachKosten {
            current: pfad[0],
            ..Default::default()
        },
        pfad.iter().map(|Kante { von, zu }| von.kosten_zu(zu)).sum(),
        pfad,
    ))
}

pub fn a1(eingabe: String) {
    let orte = eingabe
        .trim_start_matches('\u{feff}') // manchmal gibt es ein BOM am Anfang eines Strings
        .split_ascii_whitespace()
        .map(|n| n.replacen('.', "", 1).parse::<i64>().expect("Keine Zahl!"))
        .tuples()
        .enumerate()
        .map(|(i, (x, y))| Ort {
            x,
            y,
            s: 1 << i as u128,
        })
        .collect_vec();
    let kanten = orte
        .iter()
        .copied()
        .cartesian_product(orte.clone())
        .cartesian_product(orte.clone())
        .map(|((a, b), c)| (a, b, c))
        .filter(|(a, b, c)| a != b && a != c && b != c)
        .filter(|(a, b, c)| b.moegliche_abbiegung(a, c))
        .map(|(a, b, c)| (Kante { von: a, zu: b }, Kante { von: b, zu: c }))
        .into_group_map();
    if let Some(pfad) = pfad_sortieren(orte.clone(), kanten) {
        display(orte.clone(), pfad);
    }
}

#[show_image::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    loese_aufgabe(a1);
    Ok(())
}

struct Pfad2 {
    /// ohne letztes == anfang
    orte: Vec<Ort>,
    /// kantenweise => fehler_bis.len() - 2 - 1 == orte.len();
    /// - 2: drei orte == eine kante => 2 orte "weniger"
    /// - 1: letzter ort => anfangsort
    fehler_bis: Vec<u8>
}
struct Fenster {
    von: u8,
    breite: u8,
    zu: u8
}
impl Fenster {
    fn fehler_auf_pfad(&self, kanten: HashMap<Kante, u128>, pfad: &Pfad2) -> u8 {
        let neue_kosten_links = if self.zu >= 2 {pfad.fehler_bis[self.zu as usize - 2]}else{0};
        let neue_kosten_mitte = if self.breite >= 2 {pfad.fehler_bis[self.zu as usize + self.breite as usize] - pfad.fehler_bis[self.zu as usize]} else{0};
        let neue_kosten_rechts = if self.zu as usize + 2 < pfad.orte.len(){pfad.fehler_bis.last().unwrap() - pfad.fehler_bis[self.zu as usize + 2]}else{0};
        let kosten_grenze_links0 = if self.zu >= 2 && (self.breite >= 1 || self.zu as usize + self.breite as usize <= pfad.orte.len() - 1 + 1) {kanten.get(&Kante{von: pfad.orte[self.zu as usize - 2], zu: pfad.orte[self.zu as usize - 1]}).filter(|o| pfad.orte[self.von as usize].s & **o != 0).map(|n| 1).unwrap_or(0)} else {0};
        let kosten_grenze_links1 = if self.zu >= 1 && (self.breite >= 2 || self.zu as usize + self.breite as usize <= pfad.orte.len() - 1 + 1) {kanten.get(&Kante{von: pfad.orte[self.zu as usize - 1], zu: pfad.orte[self.von as usize]}).filter(|o| pfad.orte[self.von as usize + 1].s & **o != 0).map(|n| 1).unwrap_or(0)} else {0};
        let kosten_grenze_rechts0 = if self.zu as usize <= pfad.orte.len() - 1 + 1 && (self.breite >= 2  || (self.breite >= self.zu && self.zu - self.breite >= 1)){kanten.get(&Kante{von: pfad.orte[self.zu as usize + self.breite as usize - 1], zu: pfad.orte[self.zu as usize + self.breite as usize]}).filter(|o| pfad.orte[self.zu as usize + self.breite as usize + 1].s & **o != 0).map(|n| 1).unwrap_or(0)}else{0};
        let kosten_grenze_rechts1 = if self.zu as usize <= pfad.orte.len() - 1 + 1 - 1 && (self.breite >= 1  || (self.breite >= self.zu && self.zu - self.breite >= 0)){kanten.get(&Kante{von: pfad.orte[self.zu as usize + self.breite as usize], zu: pfad.orte[self.zu as usize + self.breite as usize + 1]}).filter(|o| pfad.orte[self.zu as usize + self.breite as usize + 2].s & **o != 0).map(|n| 1).unwrap_or(0)}else{0};
        neue_kosten_links + neue_kosten_mitte + neue_kosten_rechts + kosten_grenze_links0 + kosten_grenze_links1 + kosten_grenze_rechts0 + kosten_grenze_rechts1
    }
}

// Bester pfad fuer 4:
// best_pfad(orte, kanten) = Wrapper(
//     OrtNachKosten {
//         besucht: 1111111111111111111111111,
//         ort: Kante {
//             von: Ort {
//                 x: -137317503,
//                 y: -20146939,
//                 i: 14,
//             },
//             zu: Ort {
//                 x: -240369194,
//                 y: 57426131,
//                 i: 22,
//             },
//         },
//     },
//     158561921299459361,
//     [
//         Kante {
//             von: Ort {
//                 x: -240369194,
//                 y: 57426131,
//                 i: 22,
//             },
//             zu: Ort {
//                 x: 144832862,
//                 y: -43476284,
//                 i: 9,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: 144832862,
//                 y: -43476284,
//                 i: 9,
//             },
//             zu: Ort {
//                 x: 153130159,
//                 y: -20360910,
//                 i: 2,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: 153130159,
//                 y: -20360910,
//                 i: 2,
//             },
//             zu: Ort {
//                 x: -154088455,
//                 y: 115022553,
//                 i: 7,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: -154088455,
//                 y: 115022553,
//                 i: 7,
//             },
//             zu: Ort {
//                 x: -221149792,
//                 y: -32862538,
//                 i: 23,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: -221149792,
//                 y: -32862538,
//                 i: 23,
//             },
//             zu: Ort {
//                 x: -129104485,
//                 y: -155041640,
//                 i: 3,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: -129104485,
//                 y: -155041640,
//                 i: 3,
//             },
//             zu: Ort {
//                 x: 42137753,
//                 y: -60319863,
//                 i: 21,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: 42137753,
//                 y: -60319863,
//                 i: 21,
//             },
//             zu: Ort {
//                 x: 33379688,
//                 y: 100161238,
//                 i: 24,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: 33379688,
//                 y: 100161238,
//                 i: 24,
//             },
//             zu: Ort {
//                 x: -119026308,
//                 y: 168453598,
//                 i: 19,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: -119026308,
//                 y: 168453598,
//                 i: 19,
//             },
//             zu: Ort {
//                 x: -219148505,
//                 y: 103685337,
//                 i: 17,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: -219148505,
//                 y: 103685337,
//                 i: 17,
//             },
//             zu: Ort {
//                 x: -191716829,
//                 y: -28360492,
//                 i: 12,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: -191716829,
//                 y: -28360492,
//                 i: 12,
//             },
//             zu: Ort {
//                 x: 51008140,
//                 y: 5769601,
//                 i: 20,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: 51008140,
//                 y: 5769601,
//                 i: 20,
//             },
//             zu: Ort {
//                 x: 101498782,
//                 y: 33484198,
//                 i: 4,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: 101498782,
//                 y: 33484198,
//                 i: 4,
//             },
//             zu: Ort {
//                 x: 139446709,
//                 y: 233238,
//                 i: 8,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: 139446709,
//                 y: 233238,
//                 i: 8,
//             },
//             zu: Ort {
//                 x: 94789917,
//                 y: -67087689,
//                 i: 15,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: 94789917,
//                 y: -67087689,
//                 i: 15,
//             },
//             zu: Ort {
//                 x: -82864121,
//                 y: -104173600,
//                 i: 1,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: -82864121,
//                 y: -104173600,
//                 i: 1,
//             },
//             zu: Ort {
//                 x: -98760442,
//                 y: -81770618,
//                 i: 18,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: -98760442,
//                 y: -81770618,
//                 i: 18,
//             },
//             zu: Ort {
//                 x: -16723130,
//                 y: -12689542,
//                 i: 5,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: -16723130,
//                 y: -12689542,
//                 i: 5,
//             },
//             zu: Ort {
//                 x: -20971208,
//                 y: -5637107,
//                 i: 10,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: -20971208,
//                 y: -5637107,
//                 i: 10,
//             },
//             zu: Ort {
//                 x: -239848226,
//                 y: 8671399,
//                 i: 6,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: -239848226,
//                 y: 8671399,
//                 i: 6,
//             },
//             zu: Ort {
//                 x: -239414022,
//                 y: 40427118,
//                 i: 11,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: -239414022,
//                 y: 40427118,
//                 i: 11,
//             },
//             zu: Ort {
//                 x: -107988514,
//                 y: 185173669,
//                 i: 16,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: -107988514,
//                 y: 185173669,
//                 i: 16,
//             },
//             zu: Ort {
//                 x: 20212169,
//                 y: 156013261,
//                 i: 0,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: 20212169,
//                 y: 156013261,
//                 i: 0,
//             },
//             zu: Ort {
//                 x: 28913721,
//                 y: 58699880,
//                 i: 13,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: 28913721,
//                 y: 58699880,
//                 i: 13,
//             },
//             zu: Ort {
//                 x: -137317503,
//                 y: -20146939,
//                 i: 14,
//             },
//         },
//         Kante {
//             von: Ort {
//                 x: -137317503,
//                 y: -20146939,
//                 i: 14,
//             },
//             zu: Ort {
//                 x: -240369194,
//                 y: 57426131,
//                 i: 22,
//             },
//         },
//     ],
// )
