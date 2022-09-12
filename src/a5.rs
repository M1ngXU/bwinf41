use std::collections::{HashMap, HashSet, LinkedList};

use ethnum::U256;
use itertools::Itertools;

pub(crate) fn a52(graph_str: String) {
    let graph = graph_str
        .split_ascii_whitespace()
        .map(|s| s.parse::<usize>().expect("Unerwartete Zahl in Aufgabe."))
        .tuples()
        .skip(1)
        .group_by(|(von, _)| *von)
        .into_iter()
        .map(|(von, kanten)| (von, kanten.map(|(_von, nach)| nach).collect::<HashSet<_>>()))
        .collect::<HashMap<_, _>>();

    let mut sasha = HashSet::from([vec![1]]);
    let mut mika = HashSet::from([vec![2]]);
    let mut sasha_momentan = HashSet::from([1]);
    let mut mika_momentan = HashSet::from([2]);
    let mut gesehen = HashSet::new();
    let mut schritte = 0;
    while let None = sasha_momentan.intersection(&mika_momentan).next() {
        if !gesehen.insert(
            sasha_momentan
                .iter()
                .map(|n| *n)
                .chain(mika_momentan.iter().map(|n| *n + 200))
                .collect_vec(),
        ) {
            println!("Keine Loesung! {schritte}");
            return;
        }
        sasha = sasha
            .iter()
            .filter_map(|path| {
                graph.get(&path[schritte]).map(|n| {
                    n.iter().map(|mgl_kante| {
                        path.iter()
                            .copied()
                            .chain(std::iter::once(*mgl_kante))
                            .collect_vec()
                    })
                })
            })
            .flatten()
            .collect();
        sasha_momentan = sasha.iter().map(|s| *s.last().unwrap()).collect();
        mika = mika
            .iter()
            .filter_map(|path| {
                graph.get(&path[schritte]).map(|n| {
                    n.iter().map(|mgl_kante| {
                        path.iter()
                            .copied()
                            .chain(std::iter::once(*mgl_kante))
                            .collect_vec()
                    })
                })
            })
            .flatten()
            .collect();
        mika_momentan = mika.iter().map(|s| *s.last().unwrap()).collect();
        schritte += 1;
        println!("{schritte}");
    }
    println!("moeglich");
    //sasha_momentan.intersection(&mika_momentan).map(|treffpunkt| )
}

pub(crate) fn a5(graph: String) {
    let mut nachfolgende_knoten = [(U256::ZERO, U256::ZERO); 255];
    let mut vorgaenger_knoten = [U256::ZERO; 255];
    for i in 0..255 {
        nachfolgende_knoten[i].0 |= U256::ONE << i;
    }
    for (von, nach) in graph
        .split_ascii_whitespace()
        .map(|s| {
            s.parse::<usize>()
                .expect("Unerwarteter Buchstabe in Aufgabe.")
                - 1 // -1 da die knoten bei 0 beginnen
        })
        .tuples()
        .skip(1)
    // erste Zeile beschreibt die Anzahl der Knoten/Kanten, hier unwichtig
    {
        nachfolgende_knoten[von].1 |= U256::ONE << nach;
        vorgaenger_knoten[nach] |= U256::ONE << von;
    }
    let mut sasha_mgl = LinkedList::from([U256::ONE << 0]);
    let mut mika_mgl = vec![U256::ONE << 1];
    let mut gesehen = HashSet::new();
    let mut schritte = 0;
    while (sasha_mgl.front().unwrap() & mika_mgl[schritte]) == U256::ZERO {
        if !gesehen.insert((*sasha_mgl.front().unwrap(), mika_mgl[schritte])) {
            println!("Keine Loesung! {schritte}");
            return;
        }
        let mut sasha_mgl_neu = U256::ZERO;
        let mut mika_mgl_neu = U256::ZERO;
        for (knoten, kanten) in &nachfolgende_knoten {
            if (sasha_mgl.front().unwrap() & knoten) != 0 {
                sasha_mgl_neu |= kanten;
            }
            if (mika_mgl[schritte] & knoten) != 0 {
                mika_mgl_neu |= kanten;
            }
        }
        sasha_mgl.push_front(sasha_mgl_neu);
        mika_mgl.push(mika_mgl_neu);
        schritte += 1;
    }

    let overlap = sasha_mgl.front().unwrap() & mika_mgl[schritte];
    let mut pfade = to_masks(overlap)
        .map(|n| LinkedList::from([n.trailing_zeros()]))
        .collect_vec();
    for mgl in sasha_mgl.into_iter().skip(1) {
        pfade = pfade
            .into_iter()
            .flat_map(|mut p| {
                let mut vorgaenger_mgl = to_masks(vorgaenger_knoten[*p.front().unwrap() as usize] & mgl);
                let erstes = vorgaenger_mgl.next().unwrap(); // mindestens 1 Weg
                let mut rest = vorgaenger_mgl.map(|n| {
                    std::iter::once(n.trailing_zeros()).chain(p.iter().copied())
                        .collect()
                }).collect_vec();
                p.push_front(erstes.trailing_zeros());
                rest.push(p);
                rest
            })
            .collect_vec();
    }
    pfade.into_iter().for_each(|pfad| println!("{}", pfad.into_iter().map(|n| n + 1).join(" -> ")));
    println!("Moeglich nach {schritte} Schritten.");
}

fn to_masks(n: U256) -> impl Iterator<Item = U256> {
    (0..U256::BITS)
        .map(|i| U256::ONE << i)
        .zip(std::iter::repeat(n))
        .filter(|(i, n)| (n & i) != 0)
        .map(|(i, _n)| i)
}
