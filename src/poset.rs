use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::{self, Write},
};

use crate::lefschetz_complex::{Filter, cell::Cell};

// We store full a list of successors, a strict partial order.
#[derive(Debug, Clone)]
pub struct Poset<P: Ord>(pub(crate) BTreeMap<P, Vec<P>>);
pub type DepthPoset<T: Ord + Clone> = Vec<Poset<(Cell<T>, Cell<T>)>>;

impl<P: PartialEq + Eq + Ord + Clone> Poset<P> {
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

    fn transitive_closure(&mut self) {
        let elements: Vec<P> = self.elements().cloned().collect();

        while let Some((pred, succ)) = elements
            .iter()
            .flat_map(|x: &P| elements.iter().map(move |y: &P| (x, y)))
            // .inspect(|(x, y)| {
            //     println!(
            //         "x: {:?}, y: {:?}, compare(x,y): {:?}",
            //         x,
            //         y,
            //         self.compare(x, y)
            //     );
            // })
            .find(|(x, y): &(&P, &P)| {
                self.compare(x, y).is_none()
                    && elements
                        .iter()
                        // .inspect(|z| {
                        //     println!(
                        //         " z: {:?}, compare(x,z): {:?}, compare(z,y): {:?}",
                        //         z,
                        //         self.compare(x, z),
                        //         self.compare(z, y)
                        //     )
                        // })
                        .any(|z: &P| {
                            self.compare(x, z) == Some(Ordering::Less)
                                && self.compare(z, y) == Some(Ordering::Less)
                        })
            })
            .map(|(x, y)| (x.clone(), y.clone()))
        {
            // println!("APPENDING LESS(x,y), where x: {:?}, y: {:?}", pred, succ);
            self.set_less(&pred, succ);
        }
    }

    pub fn new(
        elements: impl IntoIterator<Item = P>,
        relations: impl Iterator<Item = (P, P)>,
    ) -> Self {
        let mut poset = Self(
            elements
                .into_iter()
                .map(|elem| (elem, Vec::new()))
                .collect::<BTreeMap<_, _>>(),
        );

        for (pred, succ) in relations.into_iter() {
            poset.set_less(&pred, succ);
        }

        poset.transitive_closure();

        poset
    }

    pub fn depth(&self) -> usize {
        fn dfs<P: PartialEq + Eq + Ord + Clone>(
            node: &P,
            graph: &BTreeMap<P, Vec<P>>,
            memo: &mut BTreeMap<P, usize>,
        ) -> usize {
            if let Some(&length) = memo.get(node) {
                return length;
            }

            let max_len = graph
                .get(node)
                .unwrap_or(&Vec::new())
                .iter()
                .map(|succ| dfs(succ, graph, memo))
                .max()
                .unwrap_or(0);

            memo.insert(node.clone(), max_len + 1);
            max_len + 1
        }

        let mut memo = BTreeMap::new();
        self.0
            .keys()
            .map(|node| dfs(node, &self.0, &mut memo))
            .max()
            .unwrap_or(0)
            - 1 // length = nodes - 1
    }
}

impl Poset<(usize, usize)> {
    pub fn permute(self, left_perm: &[usize], right_perm: &[usize]) -> Self {
        self.map(|(left, right)| (left_perm[left], right_perm[right]))
    }
}

///Graphviz format
impl<P: PartialEq + Eq + Ord + Clone + fmt::Display> fmt::Display for Poset<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "digraph G {{\n  rankdir=BT;")?;

        for (node, succs) in self.0.iter() {
            for succ in succs.iter() {
                writeln!(f, "  {}->{}", node, succ)?;
            }
        }

        writeln!(f, "}}")
    }
}

// pub fn display_depth_poset<T: PartialEq + Eq + Ord + Clone + Copy + fmt::Display + fmt::Debug>(
pub fn display_depth_poset<T: Ord + Clone + fmt::Display>(
    depth_poset: &DepthPoset<T>,
    filter: &Filter<T>,
) -> String {
    // println!("{:?}", depth_poset);
    let colors = [
        "#6fa8dc", "#f6b26b", "#93c47d", "#8e7cc3", "#76a5af", "#c27ba0",
    ];
    assert!(depth_poset.len() <= colors.len(), "Not enough colors");
    let mut buffer = String::new();

    writeln!(
        &mut buffer,
        r#"digraph G {{
    label = "Filter: {}";"#,
        filter
            .iter()
            .map(|cell| cell.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
    .unwrap();

    writeln!(
        &mut buffer,
        r#"rankdir=BT;
    node [shape=ellipse, style=filled, fontname="Arial"]"#
    )
    .unwrap();

    for (dim, (poset, color)) in depth_poset.iter().zip(colors.iter()).enumerate() {
        writeln!(
            &mut buffer,
            r#"
    subgraph cluster_{dim} {{
        label = "dimension {dim}, depth {}";
        style = rounded;
        subgraph {{
            node [ fillcolor="{color}" ]
            rankdir=BT;"#,
            poset.depth()
        )
        .unwrap();

        for (pred, succs) in poset.0.iter() {
            if succs.is_empty() {
                if poset
                    .elements()
                    .filter(|node| *node != pred)
                    .all(|node| poset.compare(node, pred).is_none())
                {
                    writeln!(&mut buffer, "             \"{}, {}\";", pred.0, pred.1).unwrap();
                }
            } else {
                for succ in succs.iter() {
                    writeln!(
                        &mut buffer,
                        "               \"{}, {}\" -> \"{}, {}\";",
                        pred.0, pred.1, succ.0, succ.1
                    )
                    .unwrap();
                }
            }
        }

        writeln!(
            &mut buffer,
            r#"
        }}
    }}"#
        )
        .unwrap();
    }

    writeln!(&mut buffer, "}}").unwrap();

    buffer
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn new() {
        let poset = Poset::<u32>::new(
            [1, 2, 3, 4, 5],
            [(2, 3), (3, 4), (4, 5), (1, 4)].into_iter(),
        );

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

    #[test]
    fn depth() {
        let poset = Poset::<u32>::new(
            [1, 2, 3, 4, 5],
            [(2, 3), (3, 4), (4, 5), (1, 4)].into_iter(),
        );

        assert_eq!(poset.depth(), 3);
    }
}
