use crate::{
    lefschetz_complex::{LefschetzComplex, cell::Cell},
    matrix::{Vec2d, ring::Z2},
};

pub type Complex = LefschetzComplex<&'static str, Vec2d<Z2>>;

/// We take a regular polygon with `N > 2` and glue its edges with the same orientation. In order
/// to preserve regularity, we need to break the outer edge in half.
pub fn glued_polygon() -> Complex
where
    // [(); 4 * N + 5]:,
{
    // assert!(N > 2, "This is not a polygon");
    const N: usize = 3;
    let face_relations: [(Cell<&'static str>, Cell<&'static str>); 8 * N + 4] = {
        let mut buffer = [(Cell("Dummy", 0), (Cell("Dummy", 0))); 8 * N + 4];

        buffer[0] = (Cell("a", 0), Cell("ab", 1));
        buffer[1] = (Cell("b", 0), Cell("ab", 1));
        buffer[2] = (Cell("a", 0), Cell("ba", 1));
        buffer[3] = (Cell("b", 0), Cell("ba", 1));

        buffer[4] = (Cell("o", 0), Cell("e0", 1));
        buffer[5] = (Cell("a", 0), Cell("e0", 1));
        buffer[6] = (Cell("o", 0), Cell("e1", 1));
        buffer[7] = (Cell("b", 0), Cell("e1", 1));

        buffer[8] = (Cell("o", 0), Cell("e2", 1));
        buffer[9] = (Cell("a", 0), Cell("e2", 1));
        buffer[10] = (Cell("o", 0), Cell("e3", 1));
        buffer[11] = (Cell("b", 0), Cell("e3", 1));

        buffer[12] = (Cell("o", 0), Cell("e4", 1));
        buffer[13] = (Cell("a", 0), Cell("e4", 1));
        buffer[14] = (Cell("o", 0), Cell("e5", 1));
        buffer[15] = (Cell("b", 0), Cell("e5", 1));

        buffer[16] = (Cell("e0", 1), Cell("t0", 2));
        buffer[17] = (Cell("e1", 1), Cell("t0", 2));

        buffer[18] = (Cell("e1", 1), Cell("t1", 2));
        buffer[19] = (Cell("e2", 1), Cell("t1", 2));

        buffer[20] = (Cell("e2", 1), Cell("t2", 2));
        buffer[21] = (Cell("e3", 1), Cell("t2", 2));

        buffer[22] = (Cell("e3", 1), Cell("t3", 2));
        buffer[23] = (Cell("e4", 1), Cell("t3", 2));

        buffer[24] = (Cell("e4", 1), Cell("t4", 2));
        buffer[25] = (Cell("e5", 1), Cell("t4", 2));

        buffer[26] = (Cell("e5", 1), Cell("t5", 2));
        buffer[27] = (Cell("e0", 1), Cell("t5", 2));

        buffer
    };

    for x in face_relations {
        println!("{:?}", x);
    }

    Complex::from_face_relations(face_relations)
}

#[cfg(test)]
mod test {

    #[test]
    fn glued_polygon() {
        let _ = super::glued_polygon();
    }
}
