#[derive(PartialEq, Eq, Debug)]
pub struct Permutations {
    curr: Option<Vec<usize>>,
}

pub type Permutation = Vec<usize>;
impl Permutations {
    pub fn new(size: usize) -> Self {
        Self {
            curr: Some((0..size).collect::<Permutation>()),
        }
    }
}
use std::mem::MaybeUninit;

pub fn inverse(permutation: &Permutation) -> Permutation {
    let n = permutation.len();
    let mut inverse: Permutation = {
        let mut vec: Vec<MaybeUninit<usize>> = Vec::with_capacity(n);
        unsafe {
            vec.set_len(n);
            // Transmute Vec<MaybeUninit<usize>> to Vec<usize>
            std::mem::transmute::<Vec<MaybeUninit<usize>>, Vec<usize>>(vec)
        }
    };

    for i in 0..n {
        unsafe {*inverse.get_unchecked_mut(*permutation.get_unchecked(i)) = i;}
    }

    inverse
}

impl Iterator for Permutations {
    type Item = Permutation; // Return owned Vec to avoid lifetime issues

    fn next(&mut self) -> Option<Self::Item> {
        let mut vec = self.curr.take()?; // Take ownership of the current permutation

        let result = vec.clone(); // Return this as the current permutation

        // Generate next permutation (lexicographical)
        if next_permutation(&mut vec) {
            self.curr = Some(vec);
        } else {
            self.curr = None;
        }

        Some(result)
    }
}

// Lexicographical next permutation
fn next_permutation(data: &mut [usize]) -> bool {
    let len = data.len();
    if len < 2 {
        return false;
    }

    let mut i = len - 1;
    while i > 0 && data[i - 1] >= data[i] {
        i -= 1;
    }
    if i == 0 {
        return false;
    }

    let mut j = len - 1;
    while data[j] <= data[i - 1] {
        j -= 1;
    }

    data.swap(i - 1, j);
    data[i..].reverse();
    true
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn two_elements() {
        let permutations: Vec<_> = Permutations::new(3).collect();
        assert_eq!(
            permutations,
            vec![
                vec![0, 1, 2],
                vec![0, 2, 1],
                vec![1, 0, 2],
                vec![1, 2, 0],
                vec![2, 0, 1],
                vec![2, 1, 0]
            ]
        );
    }

    #[test]
    fn inverse() {
        let permutation = vec![2,0,1];
        let inverse_p = super::inverse(&permutation);
        for i in 0..2 {
            assert_eq!(inverse_p[permutation[i]], i);
            assert_eq!(permutation[inverse_p[i]], i);
        }
    }
}
