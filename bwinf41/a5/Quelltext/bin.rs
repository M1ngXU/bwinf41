use std::{
    collections::{HashSet, LinkedList},
    convert::Infallible,
    str::FromStr,
};

use aufgaben_helfer::loese_aufgabe;
use bit_vec::BitVec;
use itertools::Itertools;

trait BitVecHelper {
    fn bit_and(&self, other: &Self) -> Self;
    fn als_leeres_duplizieren(&self) -> Self;
}
impl BitVecHelper for BitVec {
    fn bit_and(&self, other: &Self) -> Self {
        let mut a = self.clone();
        a.and(other);
        a
    }

    fn als_leeres_duplizieren(&self) -> Self {
        Self::from_elem(self.len(), false)
    }
}

struct Huepfburg {
    vorgaenger_knoten: Vec<BitVec>,
    nachfolger_knoten: Vec<BitVec>,
    sasha_erreichbare_knoten_folge: LinkedList<BitVec>,
    mika_erreichbare_knoten_folge: LinkedList<BitVec>,
    gesehene_zustaende: HashSet<(BitVec, BitVec)>,
    spruenge: usize,
    knoten: usize,
}
fn neue_erreichbare_knoten<I: IntoIterator<Item = usize>>(
    knoten: usize,
    erreichbare_knoten: I,
) -> BitVec {
    let mut neu = BitVec::from_elem(knoten, false);
    for knoten in erreichbare_knoten {
        neu.set(knoten, true);
    }
    neu
}
impl FromStr for Huepfburg {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut datei = s
            .split_ascii_whitespace()
            .map(|s| {
                s.parse::<usize>()
                    .expect("Unerwarteter Buchstabe in Aufgabe.")
            })
            .tuples();
        let (knoten, _kanten) = datei.next().unwrap();
        let mut vorgaenger_knoten = vec![neue_erreichbare_knoten(knoten, []); knoten];
        let mut nachfolger_knoten = vorgaenger_knoten.clone();

        for (von, nach) in datei.map(|(v, n)| ((v - 1), (n - 1))) {
            vorgaenger_knoten[nach].set(von, true);
            nachfolger_knoten[von].set(nach, true);
        }
        Ok(Self::new(vorgaenger_knoten, nachfolger_knoten, knoten))
    }
}
impl Huepfburg {
    fn new(
        vorgaenger_knoten: Vec<BitVec<u32>>,
        nachfolger_knoten: Vec<BitVec<u32>>,
        knoten: usize,
    ) -> Self {
        Self {
            vorgaenger_knoten,
            nachfolger_knoten,
            sasha_erreichbare_knoten_folge: LinkedList::from([neue_erreichbare_knoten(
                knoten,
                [0],
            )]),
            mika_erreichbare_knoten_folge: LinkedList::from([neue_erreichbare_knoten(knoten, [1])]),
            gesehene_zustaende: HashSet::new(),
            spruenge: 0,
            knoten,
        }
    }

    fn keine_loesung(self) {
        println!(
            "Keine Lösung! (Anzahl der Sprünge bis zu einer bekannten Situation: {})",
            self.spruenge
        );
    }

    fn keine_knoten(&self) -> BitVec {
        neue_erreichbare_knoten(self.knoten, [])
    }

    fn gleicher_erreichbarer_knoten(&self) -> bool {
        self.sasha_erreichbare_knoten()
            .bit_and(self.mika_erreichbare_knoten())
            .any()
    }
    fn get_neue_erreichbare_knoten(&self, momentane_knoten: &BitVec) -> BitVec {
        let mut neue_erreichbare_knoten = self.keine_knoten();
        momentane_knoten
            .iter()
            .enumerate()
            .filter(|(_knoten, erreichbar)| *erreichbar)
            .for_each(|(knoten, _erreichbar)| {
                neue_erreichbare_knoten.or(&self.nachfolger_knoten[knoten]);
            });
        neue_erreichbare_knoten
    }
    fn sasha_erreichbare_knoten(&self) -> &BitVec {
        self.sasha_erreichbare_knoten_folge.front().unwrap()
    }
    fn mika_erreichbare_knoten(&self) -> &BitVec {
        self.mika_erreichbare_knoten_folge.front().unwrap()
    }
    /// true wenn der Wert schon gesehen wurde
    fn versuche_merken(&mut self) -> bool {
        !self.gesehene_zustaende.insert((
            self.sasha_erreichbare_knoten().clone(),
            self.mika_erreichbare_knoten().clone(),
        ))
    }
    fn naechster_sprung(&mut self) {
        self.sasha_erreichbare_knoten_folge
            .push_front(self.get_neue_erreichbare_knoten(self.sasha_erreichbare_knoten()));
        self.mika_erreichbare_knoten_folge
            .push_front(self.get_neue_erreichbare_knoten(self.mika_erreichbare_knoten()));
        self.spruenge += 1;
    }
    fn treffpunkte(&self) -> Vec<usize> {
        self.sasha_erreichbare_knoten()
            .bit_and(self.mika_erreichbare_knoten())
            .iter()
            .positions(|b| b)
            .collect_vec()
    }
    fn get_sprungfolgen(&self, treffpunkte: &[usize]) -> Vec<(usize, String, String)> {
        let sasha = self.get_sprungfolge(&self.sasha_erreichbare_knoten_folge, treffpunkte);
        let mika = self.get_sprungfolge(&self.mika_erreichbare_knoten_folge, treffpunkte);
        treffpunkte
            .iter()
            .map(|treffpunkt| treffpunkt + 1)
            .map(|treffpunkt| {
                [&sasha, &mika]
                    .iter()
                    .map(|kind| {
                        kind.iter()
                            .find(|s| s[self.spruenge] == treffpunkt)
                            .unwrap()
                            .iter()
                            .join(" -> ")
                    })
                    .collect_tuple()
                    .map(|(sasha, mika)| (treffpunkt.to_owned(), sasha, mika))
                    .unwrap()
            })
            .collect_vec()
    }
    fn finde_weg(
        &self,
        mut weg: LinkedList<usize>,
        zuletzt_erreichbare_knoten: &BitVec,
    ) -> LinkedList<usize> {
        weg.push_front(
            self.vorgaenger_knoten[*weg.front().unwrap() as usize]
                .iter()
                .zip(zuletzt_erreichbare_knoten.iter())
                .position(|(v, m)| v && m)
                .unwrap(), // mindestens 1 Weg
        );
        weg
    }
    fn get_sprungfolge(
        &self,
        erreichbare_knoten_folge: &LinkedList<BitVec>,
        treffpunkte: &[usize],
    ) -> Vec<Vec<usize>> {
        treffpunkte
            .iter()
            .map(|erreichbarer_knoten| {
                erreichbare_knoten_folge
                    .iter()
                    .skip(1)
                    .fold(
                        LinkedList::from([*erreichbarer_knoten]),
                        |weg, zuletzt_erreichbare_knoten| {
                            self.finde_weg(weg, zuletzt_erreichbare_knoten)
                        },
                    )
                    .into_iter()
                    .map(|n| n + 1)
                    .collect_vec()
            })
            .collect_vec()
    }
}

pub(crate) fn a5(graph: String) {
    let mut huepfburg: Huepfburg = graph.parse().unwrap();

    while !huepfburg.gleicher_erreichbarer_knoten() {
        if huepfburg.versuche_merken() {
            huepfburg.keine_loesung();
            return;
        }
        huepfburg.naechster_sprung();
    }

    let treffpunkte = huepfburg.treffpunkte();
    for (end_feld, weg_sasha, weg_mika) in huepfburg.get_sprungfolgen(&treffpunkte) {
        println!(
            "So kommen Sasha & Mika zum Knoten {end_feld} in {} Sprünge:",
            huepfburg.spruenge
        );
        println!("Sasha:\n{weg_sasha}");
        println!("Mika:\n{weg_mika}");
    }
}

fn main() {
    loese_aufgabe(a5);
}
