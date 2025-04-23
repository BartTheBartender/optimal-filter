use std::collections::BTreeSet;

// We store full a "greater" relation.
#[derive(Debug)]
pub struct Poset<P: PartialEq + Eq + Ord>(BTreeSet<(P, P)>);

impl<P: PartialEq + Eq + Ord> Poset<P> {
    pub fn map<T: PartialEq + Eq + Ord, F: Fn(P) -> T>(self, f: F) -> Poset<T> {
        Poset::<T>(
            self.0
                .into_iter()
                .map(|(k, v)| (f(k), f(v)))
                .collect::<BTreeSet<_>>(),
        )
    }

    pub const fn pairs(&self) -> &BTreeSet<(P, P)> {
        &self.0
    }

    pub fn into_pairs(self) -> BTreeSet<(P, P)> {
        self.0
    }
}

impl<P: PartialEq + Eq + Ord> FromIterator<(P, P)> for Poset<P> {
    /// Performs also transitive reduction
    fn from_iter<T: IntoIterator<Item = (P, P)>>(iter: T) -> Self {
        let pairs: Vec<(P, P)> = iter.into_iter().collect();
        let mut mask = vec![true; pairs.len()];

        for (i_x, x) in pairs.iter().enumerate() {
            'intermediate: for (y, z) in itertools::iproduct!(
                pairs
                    .iter()
                    .enumerate()
                    .filter_map(|(i_y, y)| mask[i_y].then_some(y)),
                pairs
                    .iter()
                    .enumerate()
                    .filter_map(|(i_z, z)| mask[i_z].then_some(z)),
            )
            .filter(|(y, z)| *y != *z && *y != x && *z != x)
            {
                if x.0 == z.0 && x.1 == y.1 {
                    mask[i_x] = false;
                    break 'intermediate;
                }
            }
        }

        Self(
            pairs
                .into_iter()
                .enumerate()
                .filter_map(|(i_x, x)| mask[i_x].then_some(x))
                .collect::<BTreeSet<_>>(),
        )
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn from_pairs() {
        let poset = Poset::<u32>::from_iter([(2, 3), (3, 4), (4, 5), (1, 5), (2, 4), (1, 4)]);
        assert_eq!(poset.0, BTreeSet::from([(1, 4), (2, 3), (3, 4), (4, 5)]));
    }
}
