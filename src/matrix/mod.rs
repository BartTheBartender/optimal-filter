pub mod ring;
use crate::{
    matrix::ring::{Ring, Z2},
    poset::Poset,
};
use itertools::Itertools;
use std::{collections::BTreeSet, fmt};

pub trait Matrix<R: Ring>: Sized + PartialEq + Eq + Clone {
    fn from_rows<RowIt, It>(rows: It) -> Self
    where
        RowIt: ExactSizeIterator<Item = R>,
        It: ExactSizeIterator<Item = RowIt>;

    fn into_rows(&self) -> impl ExactSizeIterator<Item = impl ExactSizeIterator<Item = R>>;

    fn from_cols<ColIt, It>(cols: It) -> Self
    where
        ColIt: ExactSizeIterator<Item = R>,
        It: ExactSizeIterator<Item = ColIt>;

    fn into_cols(self) -> impl ExactSizeIterator<Item = impl ExactSizeIterator<Item = R>>;

    fn nof_rows(&self) -> usize;
    fn nof_cols(&self) -> usize;

    fn shape(&self) -> (usize, usize) {
        (self.nof_rows(), self.nof_cols())
    }

    fn zero(nof_rows: usize, nof_cols: usize) -> Self;

    fn from_rows_arr<const NOF_ROWS: usize, const ROW_LEN: usize, T: Into<R>>(
        rows: [[T; ROW_LEN]; NOF_ROWS],
    ) -> Self {
        Self::from_rows(
            rows.into_iter()
                .map(|row| row.into_iter().map(|t| t.into())),
        )
    }

    fn from_cols_arr<const NOF_COLS: usize, const COL_LEN: usize, T: Into<R>>(
        cols: [[T; COL_LEN]; NOF_COLS],
    ) -> Self {
        Self::from_cols(
            cols.into_iter()
                .map(|col| col.into_iter().map(|t| t.into())),
        )
    }

    fn row_len(&self) -> usize {
        self.nof_cols()
    }
    fn col_len(&self) -> usize {
        self.nof_rows()
    }

    /// # Safety
    /// The caller must guarantee proper bounds
    unsafe fn get_unchecked(&self, i: usize, j: usize) -> &R;

    /// # Safety
    /// The caller must guarantee proper bounds
    unsafe fn get_unchecked_mut(&mut self, i: usize, j: usize) -> &mut R;

    fn get(&self, i: usize, j: usize) -> Option<&R> {
        (i < self.nof_rows() && j < self.nof_cols()).then(|| unsafe { self.get_unchecked(i, j) })
    }

    fn get_mut(&mut self, i: usize, j: usize) -> Option<&mut R> {
        (i < self.nof_rows() && j < self.nof_cols())
            .then(|| unsafe { self.get_unchecked_mut(i, j) })
    }

    fn get_row<'a>(&'a self, i: usize) -> Option<impl Iterator<Item = &'a R>>
    where
        R: 'a,
    {
        (i < self.nof_rows())
            .then(|| (0..self.row_len()).map(move |j| unsafe { self.get_unchecked(i, j) }))
    }

    fn get_col<'a>(&'a self, j: usize) -> Option<impl Iterator<Item = &'a R>>
    where
        R: 'a,
    {
        (j < self.nof_cols())
            .then(|| (0..self.col_len()).map(move |i| unsafe { self.get_unchecked(i, j) }))
    }

    fn add_col_to_col(&mut self, src_col: usize, tgt_col: usize) -> &mut Self {
        debug_assert!(src_col != tgt_col, "Adding collumn to itself");
        debug_assert!(src_col < self.nof_cols(), "src_col out of bounds");
        debug_assert!(tgt_col < self.nof_cols(), "tgt_col out of bounds");

        for i in 0..self.nof_rows() {
            let new_value: R = *unsafe { self.get_unchecked(i, src_col) }
                + *unsafe { self.get_unchecked(i, tgt_col) };

            *unsafe { self.get_unchecked_mut(i, tgt_col) } = new_value;
        }

        self
    }

    fn add_row_to_row(&mut self, src_row: usize, tgt_row: usize) -> &mut Self {
        debug_assert!(src_row != tgt_row, "Adding rowlumn to itself");
        debug_assert!(src_row < self.nof_rows(), "src_row out of bounds");
        debug_assert!(tgt_row < self.nof_rows(), "tgt_row out of bounds");

        for j in 0..self.nof_cols() {
            let new_value: R = *unsafe { self.get_unchecked(src_row, j) }
                + *unsafe { self.get_unchecked(tgt_row, j) };

            *unsafe { self.get_unchecked_mut(tgt_row, j) } = new_value;
        }

        self
    }

    // pub fn permute_rows(self, row_perm: &Vec<usize>) -> Self {}
    // pub fn permute_cols(self, row_perm: &Vec<usize>) -> Self {}
    //
    fn permute(&self, row_perm: &Vec<usize>, col_perm: &Vec<usize>) -> Self {
        debug_assert_eq!(
            self.nof_cols(),
            row_perm.len(),
            "This is not a proper row permutation"
        );
        debug_assert_eq!(
            self.nof_cols(),
            col_perm.len(),
            "This is not a proper col permutation"
        );

        let rows = self
            .into_rows()
            .map(|row| {
                row.enumerate()
                    .map(|(idx, coeff)| (row_perm[idx], coeff))
                    .sorted_unstable_by_key(|(idx, _)| *idx)
                    .map(|(_, coeff)| coeff)
            })
            .enumerate()
            .map(|(idx, row)| (col_perm[idx], row))
            .sorted_unstable_by_key(|(idx, _)| *idx)
            .map(|(_, row)| row);

        Self::from_rows(rows)
    }

    /// Algorithm from the paper on depth posets
    #[allow(clippy::complexity, reason = "this is math")]
    fn algorithm_1(mut self) -> (BTreeSet<(usize, usize)>, BTreeSet<(usize, usize)>) {
        let mut row_mask = vec![true; self.nof_rows()];
        let mut col_mask = vec![true; self.nof_cols()];
        let mut b = BTreeSet::new();
        let mut bd_pairs = BTreeSet::new();

        while let Some((s, t)) = {
            (0..self.nof_rows())
                .rev()
                .filter(|s| row_mask[*s])
                .flat_map(|s| {
                    (0..self.nof_cols())
                        .filter(|t| col_mask[*t])
                        .map(move |t| (s, t))
                })
                .find(|(s, t)| *self.get(*s, *t).expect("this is fine here.") == R::ONE)
        } {
            println!("s: {s}, t: {t}");
            while let Some(y) = {
                (t + 1..self.nof_cols()).find(|y| {
                    col_mask[*y] && *self.get(s, *y).expect("this is fine in inner while") == R::ONE
                })
            } {
                println!(" y: {y}");
                self.add_col_to_col(t, y);
                b.insert((t, y));
            }

            bd_pairs.insert((s, t));
            row_mask[s] = false;
            col_mask[t] = false;
        }

        (bd_pairs, b)
    }

    /// Algorithm from the paper on depth posets
    #[allow(clippy::complexity, reason = "this is math")]
    fn algorithm_2(mut self) -> (BTreeSet<(usize, usize)>, BTreeSet<(usize, usize)>) {
        let mut row_mask = vec![true; self.nof_rows()];
        let mut col_mask = vec![true; self.nof_cols()];
        let mut b = BTreeSet::new();
        let mut bd_pairs = BTreeSet::new();

        while let Some((s, t)) = {
            (0..self.nof_cols())
                .filter(|t| col_mask[*t])
                .flat_map(|t| {
                    (0..self.nof_rows())
                        .rev()
                        .filter(|s| row_mask[*s])
                        .map(move |s| (s, t))
                })
                .find(|(s, t)| *self.get(*s, *t).expect("this is fine here.") == R::ONE)
        } {
            println!("s: {s}, t: {t}");
            while let Some(x) = {
                (0..s).rev().find(|x| {
                    row_mask[*x] && *self.get(*x, t).expect("this is fine in inner while") == R::ONE
                })
            } {
                println!(" x: {x}");
                self.add_row_to_row(s, x);
                b.insert((s, x));
            }

            bd_pairs.insert((s, t));
            row_mask[s] = false;
            col_mask[t] = false;
        }

        (bd_pairs, b)
    }

    ///Returns the depth poset with nodes being just pairs of indices, since we do not know their
    ///dimensions.
    fn depth_poset(self) -> Poset<(usize, usize)> {
        let (bd_pairs_1, b_1) = self.clone().algorithm_1();

        let (bd_pairs_2, b_2) = self.algorithm_2();

        debug_assert_eq!(
            bd_pairs_1, bd_pairs_2,
            "The sets of bd_pairs should be equal."
        );

        println!("B1: {:?}", b_1);
        println!("B2: {:?}", b_2);

        bd_pairs_1
            .into_iter()
            .flat_map(|x| bd_pairs_2.iter().cloned().map(move |y| (x, y)))
            .inspect(|(x, y)| {
                println!(
                    "x: {:?}, y: {:?}, b_1: {}, b_2: {}",
                    x,
                    y,
                    b_1.contains(&(x.1, y.1)),
                    b_2.contains(&(x.0, y.0))
                );
            })
            .filter(|(x, y)| b_1.contains(&(x.1, y.1)) || b_2.contains(&(x.0, y.0)))
            .collect::<Poset<_>>()
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct Vec2d<R> {
    buffer: Vec<R>,
    nof_rows: usize,
    nof_cols: usize,
}

impl<R: Ring + Copy + fmt::Debug> Matrix<R> for Vec2d<R> {
    fn from_rows<RowIt, It>(mut rows: It) -> Self
    where
        RowIt: ExactSizeIterator<Item = R>,
        It: ExactSizeIterator<Item = RowIt>,
    {
        let nof_rows = rows.len();
        assert!(
            nof_rows > 0,
            " must be positive, if the matrix is constructed from rows."
        );
        let row = rows
            .next()
            .expect("there must be at least one, if the matrix is constructed from rows.");
        let row_len = row.len();
        assert!(
            row_len > 0,
            "The row length must be positive, if the matrix is constructed from rows."
        );
        let mut buffer = Vec::with_capacity(row_len * nof_rows);
        buffer.extend(row);

        for row in rows {
            assert_eq!(row.len(), row_len, "The rows have different sizes.");
            buffer.extend(row);
        }

        Self {
            buffer,
            nof_rows,
            nof_cols: row_len,
        }
    }

    fn into_rows(&self) -> impl ExactSizeIterator<Item = impl ExactSizeIterator<Item = R>> {
        (0..self.nof_rows()).map(move |i| {
            let row_start = i * self.row_len();
            (row_start..row_start + self.row_len())
                .map(move |idx| unsafe { *self.buffer.get_unchecked(idx) })
        })
    }

    fn from_cols<ColIt, It>(mut cols: It) -> Self
    where
        ColIt: ExactSizeIterator<Item = R>,
        It: ExactSizeIterator<Item = ColIt>,
    {
        let nof_cols = cols.len();
        let col = cols
            .next()
            .expect("there must be at least one, if the matrix is constructed from rows.");
        let col_len = col.len();
        assert!(
            col_len > 0,
            "The col length must be positive, if the matrix is constructed from cols."
        );

        let mut buffer = vec![R::ZERO; nof_cols * col_len];

        col.into_iter()
            .enumerate()
            .map(|(j, coeff)| (j * nof_cols, coeff))
            .for_each(|(idx, coeff)| buffer[idx] = coeff);

        for (col, i) in cols.zip(1..col_len) {
            col.into_iter()
                .enumerate()
                .map(|(j, coeff)| (j * nof_cols + i, coeff))
                .for_each(|(idx, coeff)| buffer[idx] = coeff);
        }

        Self {
            buffer,
            nof_cols,
            nof_rows: col_len,
        }
    }

    fn into_cols(self) -> impl ExactSizeIterator<Item = impl ExactSizeIterator<Item = R>> {
        std::iter::empty::<std::iter::Empty<R>>()
    }

    fn nof_rows(&self) -> usize {
        self.nof_rows
    }

    fn nof_cols(&self) -> usize {
        self.nof_cols
    }

    unsafe fn get_unchecked(&self, i: usize, j: usize) -> &R {
        self.buffer.get_unchecked(i * self.nof_cols + j)
    }

    unsafe fn get_unchecked_mut(&mut self, i: usize, j: usize) -> &mut R {
        self.buffer.get_unchecked_mut(i * self.nof_cols + j)
    }

    fn shape(&self) -> (usize, usize) {
        (self.nof_rows(), self.nof_cols())
    }
    fn zero(nof_rows: usize, nof_cols: usize) -> Self {
        Self {
            buffer: vec![R::ZERO; nof_rows * nof_cols],
            nof_rows,
            nof_cols,
        }
    }
}

impl fmt::Display for Vec2d<Z2> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string: String = (0..self.col_len())
            .map(|i| {
                let row = (0..self.row_len())
                    .map(|j| self.get(i, j).expect("This is well-defined").to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("[{row}]")
            })
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "M({}x{})\n{string}", self.nof_rows, self.nof_cols)
    }
}

impl fmt::Debug for Vec2d<Z2> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[cfg(test)]
mod test {
    use super::{ring::Z2, *};

    #[test]
    fn from_rows() {
        assert_eq!(
            Vec2d::<Z2>::from_rows_arr([[1, 1], [0, 0],]).buffer,
            vec![Z2::ONE, Z2::ONE, Z2::ZERO, Z2::ZERO]
        );
    }

    #[test]
    fn from_cols_arr() {
        assert_eq!(
            Vec2d::<Z2>::from_cols_arr([[1, 1, 0], [1, 0, 1], [0, 1, 1]]).buffer,
            vec![
                Z2::ONE,
                Z2::ONE,
                Z2::ZERO,
                Z2::ONE,
                Z2::ZERO,
                Z2::ONE,
                Z2::ZERO,
                Z2::ONE,
                Z2::ONE
            ]
        );
    }

    #[test]
    fn get() {
        let matrix = Vec2d::<Z2>::from_rows_arr([[1, 0], [1, 0], [0, 1]]);

        assert_eq!(
            *matrix.get(0, 1).expect("This is in proper bounds."),
            Z2::ZERO
        );
        assert_eq!(
            *matrix.get(1, 0).expect("This is in proper bounds."),
            Z2::ONE
        );
        assert_eq!(
            *matrix.get(2, 1).expect("This is in proper bounds."),
            Z2::ONE
        );
        assert!(matrix.get(1, 2).is_none());
    }

    #[test]
    fn get_col_row() {
        let matrix = Vec2d::<Z2>::from_rows_arr([[1, 0], [1, 0], [0, 1]]);

        assert_eq!(
            matrix
                .get_row(0)
                .expect("This is in proper bounds.")
                .copied()
                .collect::<Vec<_>>(),
            vec![Z2::ONE, Z2::ZERO]
        );
        assert_eq!(
            matrix
                .get_col(0)
                .expect("This is in proper bounds.")
                .copied()
                .collect::<Vec<_>>(),
            vec![Z2::ONE, Z2::ONE, Z2::ZERO]
        );
    }

    #[test]
    fn permute() {
        let matrix =
            Vec2d::<Z2>::from_rows_arr([[1, 1, 1, 1], [1, 1, 0, 0], [1, 1, 1, 0], [0, 0, 1, 1]]);

        assert_eq!(
            matrix.permute(&vec![0, 1, 2, 3], &vec![3, 1, 2, 0]),
            Vec2d::<Z2>::from_rows_arr([[0, 0, 1, 1], [1, 1, 0, 0], [1, 1, 1, 0], [1, 1, 1, 1],])
        );
    }

    fn circle_from_the_paper() -> Vec2d<Z2> {
        Vec2d::from_rows_arr([
            [0, 0, 0, 0, 0, 0, 1, 1],
            [0, 0, 1, 0, 0, 1, 0, 0],
            [1, 1, 0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0, 0, 1],
            [0, 0, 1, 1, 0, 0, 0, 0],
            [0, 1, 0, 0, 1, 0, 0, 0],
            [0, 0, 0, 1, 1, 0, 0, 0],
            [0, 0, 0, 0, 0, 1, 1, 0],
        ])
    }

    #[test]
    fn algorithm_1() {
        let (bd_pairs, b) = circle_from_the_paper().algorithm_1();

        assert_eq!(
            bd_pairs,
            BTreeSet::from([(7, 5), (6, 3), (5, 1), (4, 2), (3, 0), (2, 4), (1, 6)])
        );

        assert_eq!(
            b,
            BTreeSet::from([(5, 6), (3, 4), (1, 4), (2, 4), (0, 7), (4, 7), (6, 7)])
        );
    }

    #[test]
    fn algorithm_2() {
        let (bd_pairs, b) = circle_from_the_paper().algorithm_2();

        assert_eq!(
            bd_pairs,
            BTreeSet::from([(7, 5), (6, 3), (5, 1), (4, 2), (3, 0), (2, 4), (1, 6)])
        );

        assert_eq!(
            b,
            BTreeSet::from([(3, 2), (5, 2), (4, 1), (6, 1), (2, 1), (7, 1), (1, 0)])
        );
    }
}
