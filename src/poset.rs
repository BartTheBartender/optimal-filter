use std::{cmp::Ordering, collections::BTreeMap};

// We store full a list of successors, a strict partial order.
#[derive(Debug, Clone)]
pub struct Poset<P: PartialEq + Eq + Ord>(BTreeMap<P, Vec<P>>);

impl<P: PartialEq + Eq + Ord> Poset<P> {
    pub fn map<T: PartialEq + Eq + Ord + Clone, F: Fn(P) -> T>(self, f: F) -> Poset<T> {
        Poset::<T>(
            self.0
                .into_iter()
                .map(|(node, successors)| {
                    (f(node), successors.into_iter().map(&f).collect::<Vec<_>>())
                })
                .collect::<BTreeMap<_, _>>(),
        )
    }

    pub fn compare(&self, x: &P, y: &P) -> Option<Ordering> {
        // We store a strict partial order , hence we have to check it.
        if x == y {
            Some(Ordering::Equal)
        } else {
            // Otherwise there are four cases:
            // * one of them is not in the posoet -- then they are incomparable
            // * y is a successor of x -- then x < y
            // * x is a successor of y -- then y < x
            // * they are both in the poset, but no relation -- then they are incomparable as well
            self.0
                .get(x)
                .zip(self.0.get(y))
                .and_then(|(x_succ, y_succ)| {
                    if x_succ.contains(y) {
                        Some(Ordering::Less)
                    } else if y_succ.contains(x) {
                        Some(Ordering::Greater)
                    } else {
                        None
                    }
                })
        }
    }

    pub fn elements(&self) -> impl Iterator<Item = &P> {
        self.0.keys()
    }

    /// I decided not to support manual adding relations, since it is possible to construct from
    /// pairs representing relation.
    fn set_less(&mut self, pred: &P, succ: P) {
        self.0
            .get_mut(pred)
            .expect("This should be added before")
            .push(succ)
    }
}

impl<P: PartialEq + Eq + Ord + Clone + std::fmt::Debug> FromIterator<(P, P)> for Poset<P> {
    /// Performs also transitive closure, excluding reflexive relation
    fn from_iter<T: IntoIterator<Item = (P, P)>>(iter: T) -> Self {
        let mut poset = Self(
            iter.into_iter()
                .map(|(pred, succ)| (pred, vec![succ]))
                .collect::<BTreeMap<_, _>>(),
        );

        for succ in poset
            .0
            .values()
            .flat_map(|succ| succ.iter().cloned())
            .collect::<Vec<P>>()
            .into_iter()
        {
            poset.0.entry(succ).or_default();
        }

        println!("{:?}", poset);

        let elements: Vec<P> = poset.elements().cloned().collect();

        while let Some((pred, succ)) = elements
            .iter()
            .flat_map(|x: &P| elements.iter().map(move |y: &P| (x, y)))
            .inspect(|(x, y)| {
                println!(
                    "x: {:?}, y: {:?}, compare(x,y): {:?}",
                    x,
                    y,
                    poset.compare(x, y)
                );
            })
            .find(|(x, y): &(&P, &P)| {
                poset.compare(x, y).is_none()
                    && elements
                        .iter()
                        .inspect(|z| {
                            println!(
                                " z: {:?}, compare(x,z): {:?}, compare(z,y): {:?}",
                                z,
                                poset.compare(x, z),
                                poset.compare(z, y)
                            )
                        })
                        .any(|z: &P| {
                            poset.compare(x, z) == Some(Ordering::Less)
                                && poset.compare(z, y) == Some(Ordering::Less)
                        })
            })
            .map(|(x, y)| (x.clone(), y.clone()))
        {
            println!("APPENDING LESS(x,y), where x: {:?}, y: {:?}", pred, succ);
            poset.set_less(&pred, succ);
        }

        poset
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn from_pairs() {
        let poset = Poset::<u32>::from_iter([(2, 3), (3, 4), (4, 5), (1, 4)]);

        assert_eq!(poset.compare(&1, &1), Some(Ordering::Equal));
        assert_eq!(poset.compare(&1, &2), None);
        assert_eq!(poset.compare(&1, &3), None);
        assert_eq!(poset.compare(&1, &4), Some(Ordering::Less));
        assert_eq!(poset.compare(&1, &5), Some(Ordering::Less));

        assert_eq!(poset.compare(&2, &1), None);
        assert_eq!(poset.compare(&2, &2), Some(Ordering::Equal));
        assert_eq!(poset.compare(&2, &3), Some(Ordering::Less));
        assert_eq!(poset.compare(&2, &4), Some(Ordering::Less));
        assert_eq!(poset.compare(&2, &5), Some(Ordering::Less));

        assert_eq!(poset.compare(&3, &1), None);
        assert_eq!(poset.compare(&3, &2), Some(Ordering::Greater));
        assert_eq!(poset.compare(&3, &3), Some(Ordering::Equal));
        assert_eq!(poset.compare(&3, &4), Some(Ordering::Less));
        assert_eq!(poset.compare(&3, &5), Some(Ordering::Less));

        assert_eq!(poset.compare(&4, &1), Some(Ordering::Greater));
        assert_eq!(poset.compare(&4, &2), Some(Ordering::Greater));
        assert_eq!(poset.compare(&4, &3), Some(Ordering::Greater));
        assert_eq!(poset.compare(&4, &4), Some(Ordering::Equal));
        assert_eq!(poset.compare(&4, &5), Some(Ordering::Less));

        assert_eq!(poset.compare(&5, &1), Some(Ordering::Greater));
        assert_eq!(poset.compare(&5, &2), Some(Ordering::Greater));
        assert_eq!(poset.compare(&5, &3), Some(Ordering::Greater));
        assert_eq!(poset.compare(&5, &4), Some(Ordering::Greater));
        assert_eq!(poset.compare(&5, &5), Some(Ordering::Equal));
    }
}
