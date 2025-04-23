pub mod cell;
pub mod filter;

use crate::{
    matrix::{
        Matrix,
        ring::{Ring, Z2},
    },
    poset::Poset,
};

use cell::Cell;
use filter::Filter;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

#[derive(PartialEq, Eq, Clone)]
pub struct LefschetzComplex<
    T: PartialEq + Eq + Copy + Clone + Sized + PartialOrd + Ord,
    M: Matrix<Z2>,
> {
    indices: BTreeMap<Cell<T>, Cell<usize>>,
    boundary: Vec<M>,
    dim_count: Vec<usize>,
}

impl<
    T: PartialEq + Eq + Copy + Clone + Sized + PartialOrd + Ord + fmt::Debug,
    M: Matrix<Z2> + fmt::Debug,
> LefschetzComplex<T, M>
{
    pub fn from_face_relations<FaceRelations: IntoIterator<Item = (Cell<T>, Cell<T>)>>(
        face_relations: FaceRelations,
    ) -> Self {
        let face_relations_vec = face_relations.into_iter().collect::<Vec<_>>();

        let max_dim: usize = face_relations_vec.iter().fold(0, |max_dim, (s, t)| {
            assert_eq!(s.dim() + 1, t.dim(), "This is inproper face relation.");
            std::cmp::max(max_dim, t.dim())
        });

        let mut dim_count = vec![0; max_dim + 1];

        let simplex_set =
            face_relations_vec
                .iter()
                .copied()
                .fold(BTreeSet::new(), |mut set, (s, t)| {
                    set.insert(s);
                    set.insert(t);
                    set
                });

        let mut indices = BTreeMap::new();

        for s in simplex_set {
            let s_idx = dim_count[s.dim()];
            dim_count[s.dim()] += 1;
            indices.insert(s, Cell(s_idx, s.dim()));
        }

        let mut boundary = (0..max_dim)
            .zip(1..=max_dim)
            .map(|(tgt_dim, src_dim)| (dim_count[tgt_dim], dim_count[src_dim]))
            .map(|(nof_rows, nof_cols)| M::zero(nof_rows, nof_cols))
            .collect::<Vec<_>>();

        for (s, t) in face_relations_vec.into_iter() {
            let s_ = *indices.get(&s).expect("The cell exists.");

            let t_ = *indices.get(&t).expect("The cell exists.");
            *boundary[s.dim()]
                .get_mut(t_.name(), s_.name())
                .expect("This is in proper bounds") = Z2::ONE;
        }

        Self {
            boundary,
            indices,
            dim_count,
        }
    }

    pub const fn max_dim(&self) -> usize {
        debug_assert!(
            !self.boundary.is_empty(),
            "max dim of an empty complex is undefined."
        );
        self.boundary.len()
    }

    pub(crate) fn facets_idx(&self, cell: &Cell<usize>) -> impl Iterator<Item = Cell<usize>> {
        let dim = cell.dim();

        if dim > 0 {
            self.boundary[dim - 1]
                .get_col(cell.name())
                .expect("This is well-defined.")
                .enumerate()
                .filter(|(_, coeff)| (**coeff == Z2::ONE))
                .map(move |(idx, _)| Cell(idx, dim - 1))
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            vec![].into_iter()
        }
    }

    pub(crate) fn cofacets_idx(&self, cell: &Cell<usize>) -> impl Iterator<Item = Cell<usize>> {
        let dim = cell.dim();

        if dim < self.max_dim() {
            self.boundary[dim]
                .get_row(cell.name())
                .expect("This is well-defined.")
                .enumerate()
                .inspect(|(idx, coeff)| println!("IDX: {idx}, COEFF: {coeff}"))
                .filter(|(_, coeff)| (**coeff == Z2::ONE))
                .map(move |(idx, _)| Cell(idx, dim + 1))
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            vec![].into_iter()
        }
    }

    pub fn filters<'a>(&'a self) -> impl Iterator<Item = Filter<'a, T, M>> {
        let not_added_cells = self.indices.values().copied().collect::<BTreeSet<_>>();

        Self::filters_helper(self, Filter::new(self), not_added_cells)
    }

    fn filters_helper<'a>(
        complex: &'a Self,
        partial_filter: Filter<'a, T, M>,
        not_added_cells: BTreeSet<Cell<usize>>,
    ) -> impl Iterator<Item = Filter<'a, T, M>> + 'a {
        //println!("INIT FILTERS HELPER");
        //println!("partial_filter:\n\t{partial_filter:?}");
        //println!("not_added_cells:\n\t{not_added_cells:?}");
        if not_added_cells.is_empty() {
            //println!("FINISHED");
            std::iter::once(partial_filter)
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            //println!("WORK");
            not_added_cells
                .iter()
                .filter(|cell| {
                    complex
                        .facets_idx(cell)
                        .all(|facet| !not_added_cells.contains(&facet))
                })
                .flat_map(|cell| {
                    let mut not_added_cells_ = not_added_cells.clone();
                    not_added_cells_.remove(cell);
                    let mut partial_filter_ = partial_filter.clone();
                    partial_filter_.ordering.push(*cell);
                    Self::filters_helper(complex, partial_filter_, not_added_cells_)
                })
                .collect::<Vec<_>>()
                .into_iter()
        }
    }

    pub fn filter_boundary<'a>(&'a self, filter: &Filter<'a, T, M>) -> Self {
        assert!(
            filter.complex == self,
            "The domain of the filter is incorrect"
        );

        let old_to_new = (0..=self.max_dim())
            .map(|dim| {
                filter
                    .ordering
                    .iter()
                    .filter_map(|cell| (cell.dim() == dim).then_some(cell.name()))
                    .enumerate()
                    .collect::<BTreeMap<_, _>>()
            })
            .collect::<Vec<_>>();

        let indices = self
            .indices
            .iter()
            .map(|(cell, Cell(old_name, dim))| {
                (
                    *cell,
                    Cell(
                        *old_to_new[*dim]
                            .get(old_name)
                            .expect("Renaming indices is ok."),
                        *dim,
                    ),
                )
            })
            .collect::<BTreeMap<_, _>>();

        let new_to_old = old_to_new
            .into_iter()
            .map(|old_to_new_| {
                old_to_new_
                    .into_iter()
                    .map(|(k, v)| (v, k))
                    .collect::<BTreeMap<_, _>>()
            })
            .collect::<Vec<_>>();

        let boundary = self
            .boundary
            .iter()
            .enumerate()
            .map(|(tgt_dim, boundary)| {
                let src_dim = tgt_dim + 1;
                let new_to_old_tgt = &new_to_old[tgt_dim];
                let new_to_old_src = &new_to_old[src_dim];
                M::from_rows(
                    (0..boundary.nof_cols())
                        .map(move |i_new| *new_to_old_tgt.get(&i_new).expect("i_new is ok."))
                        .map(move |i_old| {
                            (0..boundary.nof_rows())
                                .map(move |j_new| {
                                    *new_to_old_src.get(&j_new).expect("j_new is ok.")
                                })
                                .map(move |j_old| {
                                    *boundary.get(i_old, j_old).expect("boundary is ok.")
                                })
                        }),
                )
            })
            .collect::<Vec<M>>();

        Self {
            dim_count: self.dim_count.clone(),
            boundary,
            indices,
        }
    }

    /// Assuming that `self` is already filtered.
    pub fn depth_poset_idx(&self) -> Vec<Poset<(Cell<usize>, Cell<usize>)>> {
        self.boundary
            .iter()
            .enumerate()
            .map(|(dim, matrix)| {
                let (bd_pairs_1, b_1): (BTreeSet<(usize, usize)>, BTreeSet<(usize, usize)>) =
                    matrix.clone().algorithm_1();
                let (bd_pairs_2, b_2): (BTreeSet<(usize, usize)>, BTreeSet<(usize, usize)>) =
                    matrix.clone().algorithm_2();
                assert_eq!(
                    bd_pairs_1, bd_pairs_2,
                    "The bd_pairs are not identical in depth poset algorithm."
                );

                bd_pairs_1
                    .into_iter()
                    .flat_map(|x| bd_pairs_2.iter().cloned().map(move |y| (x, y)))
                    .filter(|(x, y): &((usize, usize), (usize, usize))| {
                        b_1.contains(&(x.0, y.0)) || b_2.contains(&(x.1, y.1))
                    })
                    .map(|(x, y)| {
                        (
                            (Cell(x.0, dim), Cell(x.1, dim + 1)),
                            (Cell(y.0, dim), Cell(y.1, dim + 1)),
                        )
                    })
                    .collect::<Poset<(Cell<usize>, Cell<usize>)>>()
            })
            .collect::<Vec<_>>()
    }

    /// Due to performance issued `depth_poset_idx` outputs a depth poset build with `usize`,
    /// not `T`. This function gives correct labels.
    pub fn label_depth_poset(
        &self,
        depth_poset_idx: Vec<Poset<(Cell<usize>, Cell<usize>)>>,
    ) -> Vec<Poset<(Cell<T>, Cell<T>)>> {
        let reverse = self
            .indices
            .iter()
            .map(|(k, v)| (v, k))
            .collect::<BTreeMap<_, _>>();

        depth_poset_idx
            .into_iter()
            .map(|poset| {
                poset.map(|(x, y)| {
                    (
                        **reverse.get(&x).expect("The cell is in the proper bounds."),
                        **reverse.get(&y).expect("The cell is in the proper bounds."),
                    )
                })
            })
            .collect::<Vec<_>>()
    }

    // /// Returns all the possible depth posets. For now computes them using all possible filters.
    // pub fn all_depth_poset_idx(
    //     &self,
    // ) -> impl Iterator<Item = Vec<Poset<(Cell<usize>, Cell<usize>)>>> {
    //     self.filters().map(|filter|
    // }
}

impl<
    T: PartialEq + Eq + Copy + Clone + fmt::Debug + fmt::Display + Sized + PartialOrd + Ord,
    M: Matrix<Z2> + fmt::Debug,
> fmt::Debug for LefschetzComplex<T, M>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (cell, Cell(idx, _)) in &self.indices {
            writeln!(f, "idx {idx}, {cell:?}")?;
        }

        for matrix in &self.boundary {
            writeln!(f, "{matrix:?}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::matrix::{Vec2d, ring::Z2};

    #[test]
    fn from_face_relations() {
        let complex = LefschetzComplex::<&'static str, Vec2d<Z2>>::from_face_relations([
            (Cell("a", 0), Cell("ab", 1)),
            (Cell("b", 0), Cell("ab", 1)),
            (Cell("a", 0), Cell("ac", 1)),
            (Cell("c", 0), Cell("ac", 1)),
            (Cell("b", 0), Cell("bc", 1)),
            (Cell("c", 0), Cell("bc", 1)),
        ]);

        assert_eq!(complex.boundary.len(), 1);
        let matrix = Vec2d::<Z2>::from_rows_arr([
            [Z2::ONE, Z2::ONE, Z2::ZERO],
            [Z2::ONE, Z2::ZERO, Z2::ONE],
            [Z2::ZERO, Z2::ONE, Z2::ONE],
        ]);

        assert_eq!(complex.boundary[0], matrix);
    }

    #[test]
    fn facets_cofacets() {
        let complex = LefschetzComplex::<&'static str, Vec2d<Z2>>::from_face_relations([
            (Cell("a", 0), Cell("ab", 1)),
            (Cell("b", 0), Cell("ab", 1)),
            (Cell("a", 0), Cell("ac", 1)),
            (Cell("c", 0), Cell("ac", 1)),
            (Cell("b", 0), Cell("bc", 1)),
            (Cell("c", 0), Cell("bc", 1)),
        ]);

        assert_eq!(
            complex.facets_idx(&Cell(2, 1)).collect::<Vec<_>>(),
            vec![Cell(1, 0), Cell(2, 0)]
        );

        assert_eq!(
            complex.cofacets_idx(&Cell(0, 0)).collect::<Vec<_>>(),
            vec![Cell(0, 1), Cell(1, 1)]
        );
    }

    #[test]
    #[ignore]
    fn filter_boundary() {
        let complex = LefschetzComplex::<&'static str, Vec2d<Z2>>::from_face_relations([
            (Cell("a", 0), Cell("ab", 1)),
            (Cell("b", 0), Cell("ab", 1)),
            (Cell("a", 0), Cell("ac", 1)),
            (Cell("c", 0), Cell("ac", 1)),
            (Cell("b", 0), Cell("bc", 1)),
            (Cell("c", 0), Cell("bc", 1)),
        ]);

        let matrix = complex
            .filter_boundary(&Filter::from_ordering(
                &complex,
                [
                    Cell("a", 0),
                    Cell("c", 0),
                    Cell("ac", 1),
                    Cell("b", 0),
                    Cell("bc", 1),
                    Cell("ab", 1),
                ],
            ))
            .boundary[0]
            .clone();

        assert_eq!(matrix, complex.boundary[0]);

        println!("{:?}", matrix);
        todo!()
    }
}
