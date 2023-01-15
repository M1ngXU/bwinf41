use bit_vec::BitVec;
use image::{ImageBuffer, Rgb};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use show_image::create_window;
use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    fmt::{Display, Formatter},
    hash::Hash,
    io::Write,
    ops::Deref,
};

use aufgaben_helfer::loese_aufgabe;
use imageproc::drawing::{draw_hollow_circle_mut, draw_line_segment_mut};
use itertools::Itertools;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Ort {
    x: i64,
    y: i64,
    i: usize,
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
        skalarprodukt >= 0
    }

    fn nachfolger_orte(&self, von: &Self, offene_orte: &Vec<Self>) -> Vec<Self> {
        offene_orte
            .iter()
            .filter(|zu| self.moegliche_abbiegung(von, zu))
            .copied()
            .collect()
    }

    fn kosten_zu(&self, zu: &Self) -> u64 {
        ((self.x - zu.x).pow(2) + (self.y - zu.y).pow(2)) as u64
    }
}

#[derive(Clone, Hash, Debug, PartialEq, Eq)]
struct OrtNachKosten {
    besucht: BitVec,
    ort: Kante,
}
#[derive(Clone, Hash, Debug)]
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

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
struct Kante {
    von: Ort,
    zu: Ort,
}

pub fn a1(eingabe: String) {
    let orte = eingabe
        .trim_start_matches('\u{feff}') // manchmal gibt es ein BOM am Anfang eines Strings
        .split_ascii_whitespace()
        .map(|n| n.replacen('.', "", 1).parse::<i64>().expect("Keine Zahl!"))
        .tuples()
        .enumerate()
        .map(|(i, (x, y))| Ort { x, y, i })
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
    let bester_pfad = kanten
        .clone()
        .into_par_iter()
        .find_map_any(|(start, nachfolger)| {
            let mut warteschlange = BinaryHeap::<Wrapper>::new();
            let mut gesehen = HashSet::<OrtNachKosten>::new();
            let mut besucht = BitVec::from_elem(orte.len(), false);
            besucht.set(start.zu.i, true);
            warteschlange.push(Wrapper(
                OrtNachKosten {
                    besucht,
                    ort: start,
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
                if naechster.besucht.all() {
                    assert_eq!(naechster.2[0].von, naechster.2[orte.len() - 1].zu);
                    return Some(naechster);
                }
                for naechster_ort in kanten
                    .get(&naechster.ort)
                    .into_iter()
                    .flatten()
                    .filter(|k| {
                        !naechster.besucht[k.zu.i]
                            && (k.zu != start.von
                                || naechster.besucht.iter().filter(|x| !*x).count() == 1)
                    })
                {
                    let mut besucht = naechster.besucht.clone();
                    besucht.set(naechster_ort.zu.i, true);
                    let mut pfad = naechster.2.clone();
                    pfad.push(*naechster_ort);
                    warteschlange.push(Wrapper(
                        OrtNachKosten {
                            ort: *naechster_ort,
                            besucht,
                        },
                        naechster.1 + naechster.ort.zu.kosten_zu(&naechster_ort.von),
                        pfad,
                    ))
                }
            }
            None
        })
        //.max_by_key(|n| n.1)
        .unwrap();
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

#[show_image::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    loese_aufgabe(a1);
    Ok(())
}
