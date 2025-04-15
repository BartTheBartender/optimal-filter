pub mod ring;
use ring::{Ring, Z2};
use std::{collections::BTreeSet, fmt};

pub trait Matrix<R: Ring>: Sized + PartialEq + Eq + Clone {
    fn from_rows<RowIt, It>(rows: It) -> Self
    where
        RowIt: ExactSizeIterator<Item = R>,
        It: ExactSizeIterator<Item = RowIt>;

    fn nof_rows(&self) -> usize;
    fn nof_cols(&self) -> usize;

    fn shape(&self) -> (usize, usize) {
        (self.nof_rows(), self.nof_cols())
    }

    fn zero(nof_rows: usize, nof_cols: usize) -> Self;

    fn from_rows_arr<const NOF_ROWS: usize, const ROW_LEN: usize>(
        rows: [[R; ROW_LEN]; NOF_ROWS],
    ) -> Self {
        Self::from_rows(rows.into_iter().map(|row| row.into_iter()))
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
            while let Some(y) = {
                (t + 1..self.nof_cols()).find(|y| {
                    col_mask[*y] && *self.get(s, *y).expect("this is fine in inner while") == R::ONE
                })
            } {
                self.add_col_to_col(t, y);
                b.insert((t, y));
            }

            bd_pairs.insert((s, t));
            row_mask[s] = false;
            col_mask[t] = false;
        }

        (b, bd_pairs)
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
                //.inspect(|(s, t)| println!("s:{s}, t:{t}"))
                .find(|(s, t)| *self.get(*s, *t).expect("this is fine here.") == R::ONE)
        } {
            while let Some(x) = {
                (0..s).rev().find(|x| {
                    row_mask[*x] && *self.get(*x, t).expect("this is fine in inner while") == R::ONE
                })
            } {
                self.add_row_to_row(s, x);
                b.insert((s, x));
            }

            bd_pairs.insert((s, t));
            row_mask[s] = false;
            col_mask[t] = false;
        }

        (bd_pairs, b)
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct Vec2d<R> {
    buffer: Vec<R>,
    nof_rows: usize,
    nof_cols: usize,
}

impl<R: Ring> Matrix<R> for Vec2d<R> {
    fn zero(nof_rows: usize, nof_cols: usize) -> Self {
        Self {
            buffer: vec![R::ZERO; nof_rows * nof_cols],
            nof_rows,
            nof_cols,
        }
    }

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

        let mut buffer = rows.next().unwrap().collect::<Vec<_>>();

        let row_len = buffer.len();
        assert!(
            row_len > 0,
            "The row length must be positive, if the matrix is constructed from rows."
        );

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
            Vec2d::from_rows_arr([[Z2(true), Z2(true)], [Z2(false), Z2(false)],]).buffer,
            vec![Z2(true), Z2(true), Z2(false), Z2(false)]
        );
    }

    #[test]
    fn get() {
        let matrix = Vec2d::from_rows_arr([
            [Z2(true), Z2(false)],
            [Z2(true), Z2(false)],
            [Z2(false), Z2(true)],
        ]);

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
        let matrix = Vec2d::from_rows_arr([
            [Z2(true), Z2(false)],
            [Z2(true), Z2(false)],
            [Z2(false), Z2(true)],
        ]);

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
    fn algorithm_1() {
        let matrix = Vec2d::from_rows_arr([
            [
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
                Z2::ZERO,
                Z2::ZERO,
                Z2::ZERO,
            ],
            [
                Z2::ONE,
                Z2::ONE,
                Z2::ZERO,
                Z2::ZERO,
                Z2::ZERO,
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
            ],
            [
                Z2::ZERO,
                Z2::ZERO,
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
            ],
        ]);
        let (b, bd_pairs) = matrix.algorithm_1();

        assert_eq!(bd_pairs, BTreeSet::from([(1, 0), (2, 2)]));
        assert_eq!(
            b,
            BTreeSet::from([
                (0, 1),
                (0, 5),
                (0, 6),
                (0, 7),
                (2, 3),
                (2, 4),
                (2, 5),
                (2, 6),
                (2, 7)
            ])
        );
    }

    #[test]
    fn algorithm_2() {
        let matrix = Vec2d::from_rows_arr([
            [
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
                Z2::ZERO,
                Z2::ZERO,
                Z2::ZERO,
            ],
            [
                Z2::ONE,
                Z2::ONE,
                Z2::ZERO,
                Z2::ZERO,
                Z2::ZERO,
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
            ],
            [
                Z2::ZERO,
                Z2::ZERO,
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
                Z2::ONE,
            ],
        ]);

        let (bd_pairs, b) = matrix.algorithm_2();

        assert_eq!(bd_pairs, BTreeSet::from([(1, 0), (2, 2)]));
        assert_eq!(b, BTreeSet::from([(1, 0), (2, 0)]));
    }
}
