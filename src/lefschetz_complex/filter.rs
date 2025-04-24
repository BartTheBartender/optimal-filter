use crate::{
    lefschetz_complex::{LefschetzComplex, cell::Cell},
    matrix::{Matrix, ring::Z2},
};
use std::{collections::BTreeMap, fmt};

#[derive(PartialEq, Eq, Clone)]
pub struct Filter<'a, T: PartialEq + Eq + Copy + Clone + PartialOrd + Ord, M: Matrix<Z2>> {
    pub(crate) complex: &'a LefschetzComplex<T, M>,
    pub(crate) ordering: Vec<Cell<usize>>,
}

impl<'a, T: PartialEq + Eq + Copy + Clone + PartialOrd + Ord, M: Matrix<Z2>> Filter<'a, T, M> {
    pub fn from_ordering_idx<Ordering: IntoIterator<Item = Cell<usize>>>(
        complex: &'a LefschetzComplex<T, M>,
        ordering_: Ordering,
    ) -> Self {
        let ordering = ordering_.into_iter().collect::<Vec<_>>();
        assert_eq!(
            complex.indices.len(),
            ordering.len(),
            "The filter is not bijective"
        );
        Self { complex, ordering }
    }

    pub fn from_ordering<Ordering: IntoIterator<Item = Cell<T>>>(
        complex: &'a LefschetzComplex<T, M>,
        ordering: Ordering,
    ) -> Self {
        Self::from_ordering_idx(
            complex,
            ordering.into_iter().map(|cell| {
                *complex
                    .indices
                    .get(&cell)
                    .expect("This cell should belong to the complex.")
            }),
        )
    }

    pub const fn new(complex: &'a LefschetzComplex<T, M>) -> Self {
        Self {
            complex,
            ordering: Vec::new(),
        }
    }
}

impl<'a, T: PartialEq + Eq + Copy + Clone + PartialOrd + Ord + fmt::Debug, M: Matrix<Z2>> fmt::Debug
    for Filter<'a, T, M>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Filter:")?;
        let reverse = self
            .complex
            .indices
            .iter()
            .map(|(k, v)| (v, k))
            .collect::<BTreeMap<_, _>>();
        for cell in &self.ordering {
            write!(f, " {}", reverse.get(cell).expect("This cell exists"))?;
        }

        Ok(())
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::matrix::Vec2d;
//     #[test]
//     #[ignore]
//     fn triangle() {
//         let complex = LefschetzComplex::<&'static str, Vec2d<Z2>>::from_face_relations([
//             (Cell("a", 0), Cell("ab", 1)),
//             (Cell("b", 0), Cell("ab", 1)),
//             (Cell("a", 0), Cell("ac", 1)),
//             (Cell("c", 0), Cell("ac", 1)),
//             (Cell("b", 0), Cell("bc", 1)),
//             (Cell("c", 0), Cell("bc", 1)),
//         ]);
//
//         for filter in complex.filters() {
//             println!("{:?}", filter);
//             complex.filter_boundary(&filter);
//         }
//
//         todo!()
//     }
// }
