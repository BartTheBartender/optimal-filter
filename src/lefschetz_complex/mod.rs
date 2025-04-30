pub mod cell;
pub mod examples;
// pub mod filter;

use crate::{
    lefschetz_complex::cell::Cell,
    matrix::{
        Matrix,
        ring::{Ring, Z2},
    },
    permutations::{Permutation, Permutations, inverse},
    poset::{DepthPoset, Poset},
    wrapper::Wrapper,
};

use itertools::{self, Itertools};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::Arc,
};

#[derive(PartialEq, Eq, Clone)]
pub struct LefschetzComplex<T: Clone + Sized + Ord, M: Matrix<Z2>> {
    pub indices: BTreeMap<Cell<T>, Cell<usize>>,
    pub boundary: Vec<M>,
    pub dim_count: Vec<usize>,
}
pub type Filter<T: Clone + Ord> = Vec<Cell<T>>;

impl<T: Clone + Ord + fmt::Display, M: Matrix<Z2> + fmt::Debug> LefschetzComplex<T, M> {
    /// relations (s,t) where s is a facet of t
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
                .cloned()
                .fold(BTreeSet::new(), |mut set, (s, t)| {
                    set.insert(s);
                    set.insert(t);
                    set
                });

        let mut indices = BTreeMap::new();

        for s in simplex_set {
            let s_idx = dim_count[s.dim()];
            let s_clone = s.clone();
            dim_count[s.dim()] += 1;
            indices.insert(s_clone, Cell(s_idx, s.dim()));
        }

        let mut boundary = (0..max_dim)
            .zip(1..=max_dim)
            .map(|(tgt_dim, src_dim)| (dim_count[tgt_dim], dim_count[src_dim]))
            .map(|(nof_rows, nof_cols)| M::zero(nof_rows, nof_cols))
            .collect::<Vec<_>>();

        println!("[ setting face relations: ] ");
        for (s, t) in face_relations_vec.into_iter() {
            println!("[ s: {s}, t: {t} ]");
            let s_ = indices.get(&s).expect("The cell exists.").clone();

            let t_ = indices.get(&t).expect("The cell exists.").clone();
            // println!(
            //     "Indices s_: {s_}, t_: {t_}. Shape of the boundary: matrix {:?}.",
            //     boundary[s.dim()].shape()
            // );
            *boundary[s.dim()]
                .get_mut(*s_.name(), *t_.name())
                .expect("This is in proper bounds") = Z2::ONE;
        }
        println!("[ done setting face relations ] ");

        Self {
            boundary,
            indices,
            dim_count,
        }
    }

    fn reverse(&self) -> BTreeMap<Cell<usize>, Cell<T>> {
        self.indices
            .iter()
            .map(|(k, v)| (v.clone(), k.clone()))
            .collect::<BTreeMap<_, _>>()
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
                .get_col(*cell.name())
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
                .get_row(*cell.name())
                .expect("This is well-defined.")
                .enumerate()
                // .inspect(|(idx, coeff)| println!("IDX: {idx}, COEFF: {coeff}"))
                .filter(|(_, coeff)| (**coeff == Z2::ONE))
                .map(move |(idx, _)| Cell(idx, dim + 1))
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            vec![].into_iter()
        }
    }

    pub fn all_permutations(&self) -> Vec<Vec<Arc<Permutation>>> {
        println!("[ computing permutation pairs ]");
        let all_permutations: Vec<Vec<Arc<Permutation>>> = self
            .dim_count
            .iter()
            .cloned()
            .map(|dim| {
                Permutations::new(dim)
                    .map(|permutation: Permutation| Arc::new(permutation))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        println!("[ permutations computed ]");

        all_permutations
    }
}

pub type PermIdx = usize;
pub type UsizePoset = Poset<(usize, usize)>; // represents the depth posets with elements being
// persistence pairs, It must be stored in a vector since it does not know the dimensions
impl<T: Clone + Ord + fmt::Display + Send + Sync, M: Matrix<Z2> + fmt::Debug>
    LefschetzComplex<T, M>
{
    fn depth_posets_in_dimensions<W: Wrapper<UsizePoset>>(
        &self,
        all_permutations: &Vec<Vec<Arc<Permutation>>>,
    ) -> Vec<FxHashMap<(PermIdx, PermIdx), W>> {
        println!("[ computing depth posets in dimensions ]");
        let depth_posets_in_dimensions: Vec<FxHashMap<(PermIdx, PermIdx), W>> = self
            .boundary
            .iter()
            .zip_eq(all_permutations.iter().zip(all_permutations.iter().skip(1)))
            .par_bridge()
            // .inspect(|(matrix, _)| {
            //     println!("[ filtering the matrix\n\n{:?}\n\n]", matrix);
            // })
            .map(|(matrix, (row_perms, col_perms))| {
                // note that the depth poset is computed for the permuted coordinates
                let row_perms_inv = row_perms
                    .iter()
                    .map(|perm| inverse(perm))
                    .collect::<Vec<_>>();
                let col_perms_inv = col_perms
                    .iter()
                    .map(|perm| inverse(perm))
                    .collect::<Vec<_>>();
                itertools::iproduct!(0..row_perms.len(), 0..col_perms.len())
                    .par_bridge()
                    .map(|(row_perm_idx, col_perm_idx)| -> ((usize, usize), W) {
                        // println!(
                        //     "# examining {row_perm_idx} {col_perm_idx} : {:?} {:?} for {:?}",
                        //     row_perms[row_perm_idx].as_ref(),
                        //     col_perms[col_perm_idx].as_ref(),
                        //     matrix
                        // );
                        (
                            (row_perm_idx, col_perm_idx),
                            W::from_object(
                                matrix
                                    .permute(
                                        row_perms[row_perm_idx].as_ref(),
                                        col_perms[col_perm_idx].as_ref(),
                                    )
                                    .depth_poset()
                                    .permute(
                                        &row_perms_inv[row_perm_idx],
                                        &col_perms_inv[col_perm_idx],
                                    ),
                            ),
                        )
                    })
                    // .inspect(|(perm_idx, wrapper)| {
                    //     println!("Result of computation: {perm_idx:?}, {wrapper:?}");
                    // })
                    .collect()
            })
            .collect();

        println!("[ depth posets in dimensions computed ]");

        depth_posets_in_dimensions
    }

    pub fn depth_poset_min_depth(&self) -> (Filter<T>, DepthPoset<T>) {
        #[derive(Clone, Debug)]
        struct DepthWrapper {
            poset: UsizePoset,
            depth: usize,
        }
        impl Wrapper<UsizePoset> for DepthWrapper {
            type Info = usize;

            fn info(&self) -> Self::Info {
                self.depth
            }

            fn into_object(self) -> UsizePoset {
                self.poset
            }

            fn from_object(object: UsizePoset) -> Self {
                let depth = object.depth();
                Self {
                    poset: object,
                    depth,
                }
            }
        }

        fn depth_poset_fixed_depth(
            depth_posets_in_dimensions: &Vec<FxHashMap<(PermIdx, PermIdx), DepthWrapper>>,
            all_permutations_idx: &Vec<Vec<PermIdx>>,
            fixed_depth: usize,
        ) -> Option<(Vec<PermIdx>, Vec<UsizePoset>)> {
            println!("[ seeking depth poset with minimal depth = {fixed_depth} ]");
            // We filter the depth posets which are of depth at most fixed depth
            let depth_posets_in_dimensions_good_depth: Vec<
                FxHashMap<(PermIdx, PermIdx), UsizePoset>,
            > = depth_posets_in_dimensions
                .iter()
                .map(|fxhashmap| {
                    fxhashmap
                        .iter()
                        .filter(|(_, wrapper)| (wrapper.info() <= fixed_depth))
                        .map(|(idx, wrapper)| (*idx, wrapper.clone().into_object()))
                        .collect()
                })
                .collect();

            // we seek such a permutation such that its depth poset is of depth <= fixed_depth. It
            // will be consttruced recursively, by adding resp. permutations in appropriate
            // dimensions and quitting whenever possible
            let good_depth_permutation: Option<Vec<PermIdx>> = {
                let nof_dims = all_permutations_idx.len();
                let mut partial_result = Vec::<PermIdx>::with_capacity(nof_dims);

                fn extend(
                    partial_result: &mut Vec<PermIdx>,
                    nof_dims: usize,
                    all_permutations_idx: &Vec<Vec<PermIdx>>,
                    depth_posets_in_dimensions_good_depth: &Vec<
                        FxHashMap<(PermIdx, PermIdx), UsizePoset>,
                    >,
                ) -> bool {
                    let curr_idx = partial_result.len();
                    // println!(
                    //     " [extend with curr_idx : {curr_idx}, partial_result: {partial_result:?} ]"
                    // );

                    if curr_idx == 0 {
                        for perm in all_permutations_idx
                            .first()
                            .expect("We have to fucking start somewhere, right?")
                        {
                            partial_result.push(*perm);
                            if extend(
                                partial_result,
                                nof_dims,
                                all_permutations_idx,
                                depth_posets_in_dimensions_good_depth,
                            ) {
                                return true;
                            } else {
                                partial_result.pop();
                            }
                        }
                        // println!(" [ failed to construct the permutation ] ");
                        false
                    } else if curr_idx == nof_dims {
                        true
                    } else {
                        let last_perm: PermIdx = *partial_result
                            .last()
                            .expect("We have fucked started, right?");
                        for perm in all_permutations_idx
                            .get(curr_idx)
                            .expect("We have to fucking start somewhere, right?")
                            .iter()
                            .filter(|next_perm| {
                                depth_posets_in_dimensions_good_depth
                                    .get(curr_idx.strict_sub(1))
                                    .expect("The curr_idx is in proper funcking bounds")
                                    .get(&(last_perm, **next_perm))
                                    .is_some()
                            })
                        {
                            partial_result.push(*perm);
                            if extend(
                                partial_result,
                                nof_dims,
                                all_permutations_idx,
                                depth_posets_in_dimensions_good_depth,
                            ) {
                                return true;
                            } else {
                                partial_result.pop();
                            }
                        }
                        false
                    }
                }

                extend(
                    &mut partial_result,
                    nof_dims,
                    all_permutations_idx,
                    &depth_posets_in_dimensions_good_depth,
                )
                .then_some(partial_result)
            };

            good_depth_permutation.map(|perm_vec| {
                let posets_usize: Vec<UsizePoset> = perm_vec
                    .iter()
                    .zip(perm_vec.iter().skip(1))
                    .zip(depth_posets_in_dimensions_good_depth.iter())
                    .map(|((row_perm, col_perm), hashmap)| {
                        // println!("#######\nhashmap:{hashmap:?}\n#######");
                        hashmap
                            .get(&(*row_perm, *col_perm))
                            .expect("I have funcking constructed it this way, right?")
                            .clone()
                    })
                    .collect();

                // println!("# indices of the best permutations: {perm_vec:?}");

                (perm_vec, posets_usize)
            })
        }

        let all_permutations: Vec<Vec<Arc<Vec<usize>>>> = self.all_permutations();
        let depth_posets_in_dimensions = self.depth_posets_in_dimensions(&all_permutations);
        let all_permutations_idx: Vec<Vec<PermIdx>> = all_permutations
            .iter()
            .map(|perm_vec| (0..perm_vec.len()).collect())
            .collect();

        let (perm_vec_idx, posets_usize): (Vec<PermIdx>, Vec<UsizePoset>) = (0..usize::MAX)
            .find_map(|fixed_depth| {
                depth_poset_fixed_depth(
                    &depth_posets_in_dimensions,
                    &all_permutations_idx,
                    fixed_depth,
                )
            })
            .expect("There must be fucking maximal depth");

        let reverse = self.reverse();
        let filter: Filter<T> = perm_vec_idx
            .into_iter()
            .enumerate()
            .flat_map(|(dim, perm_idx)| {
                all_permutations
                    .get(dim)
                    .expect("Now i want my permutations of fixed dim back")
                    .get(perm_idx)
                    .expect("gimme my permutation")
                    .iter()
                    .copied()
                    .map(move |idx| Cell(idx, dim))
            })
            .map(|cell| {
                reverse
                    .get(&cell)
                    .expect("Its fucking here, right?")
                    .clone()
            })
            .collect();

        let depth_poset: DepthPoset<T> = posets_usize
            .into_iter()
            .enumerate()
            .map(|(dim_birth, poset)| self.label_usize_poset(poset, dim_birth))
            .collect();

        (filter, depth_poset)
    }

    /// util function to get from poset on persistence pairs and known dimension a poset on cells.
    pub fn label_usize_poset(
        &self,
        usize_poset: UsizePoset,
        dim_birth: usize,
    ) -> Poset<(Cell<T>, Cell<T>)> {
        let reverse = self.reverse();

        usize_poset.map(|(birth, death)| {
            (
                reverse
                    .get(&Cell(birth, dim_birth))
                    .expect("The birth cell is in the complex")
                    .clone(),
                reverse
                    .get(&Cell(death, dim_birth + 1))
                    .expect("The death cell is in the complex")
                    .clone(),
            )
        })
    }

    // fn label_depth_posets(
    //     &self,
    //     all_depth_posets_idx: impl ParallelIterator<Item = Vec<UsizePoset>>,
    // ) -> impl ParallelIterator<Item = Vec<Poset<(Cell<T>, Cell<T>)>>> {
    //     let reverse = self.reverse();
    //     all_depth_posets_idx.map(move |depth_poset| {
    //         depth_poset
    //             .into_iter()
    //             .enumerate()
    //             .map(|(dim, poset)| {
    //                 poset.map(|(idx1, idx2)| {
    //                     (
    //                         reverse
    //                             .get(&Cell(idx1, dim))
    //                             .expect("This cell is in the complex")
    //                             .clone(),
    //                         reverse
    //                             .get(&Cell(idx2, dim.saturating_add(1)))
    //                             .expect("This cell is in the complex")
    //                             .clone(),
    //                     )
    //                 })
    //             })
    //             .collect::<Vec<_>>()
    //     })
    // }
}

impl<T: Clone + fmt::Display + Sized + Ord, M: Matrix<Z2> + fmt::Debug> fmt::Debug
    for LefschetzComplex<T, M>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reverse = self.reverse();
        writeln!(f, "[ Complex: ")?;
        for (dim, matrix) in self.boundary.iter().enumerate() {
            writeln!(f, "-------------------------")?;
            writeln!(f, "dim {dim}:")?;

            write!(f, "row labels: ")?;
            for row_idx in 0..matrix.nof_rows() {
                write!(
                    f,
                    " {}",
                    reverse
                        .get(&Cell(row_idx, dim))
                        .expect("The row cell belongs to the complex")
                )?;
            }
            writeln!(f, "\n")?;

            write!(f, "col labels: ")?;
            for col_idx in 0..matrix.nof_cols() {
                write!(
                    f,
                    " {}",
                    reverse
                        .get(&Cell(col_idx, dim + 1))
                        .expect("The col cell belongs to the complex")
                )?;
            }
            writeln!(f, "\n")?;

            writeln!(f, "{:?}", matrix)?;

            writeln!(f, "-------------------------")?;
        }

        writeln!(f, "end of the complex ]")?;
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

    // #[test]
    // fn all_depth_posets_idx() {
    //     let complex = LefschetzComplex::<&'static str, Vec2d<Z2>>::from_face_relations([
    //         (Cell("a", 0), Cell("ab", 1)),
    //         (Cell("b", 0), Cell("ab", 1)),
    //         (Cell("a", 0), Cell("ac", 1)),
    //         (Cell("c", 0), Cell("ac", 1)),
    //         (Cell("b", 0), Cell("bc", 1)),
    //         (Cell("c", 0), Cell("bc", 1)),
    //     ]);
    //
    //     assert_eq!(
    //         complex.all_depth_posets_idx::<TrivialWrapper<_>>().count(),
    //         36
    //     );
    // }
}
