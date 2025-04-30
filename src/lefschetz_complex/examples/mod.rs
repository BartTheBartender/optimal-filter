use super::{LefschetzComplex, cell::Cell};
use crate::matrix::{Vec2d, ring::Z2};

pub type Complex = LefschetzComplex<String, Vec2d<Z2>>;

pub fn rp2() -> Complex {

    let face_relations = [
        (Cell(String::from("v"), 0), Cell(String::from("a"), 1)),
        (Cell(String::from("w"), 0), Cell(String::from("a"), 1)),
        (Cell(String::from("v"), 0), Cell(String::from("b"), 1)),
        (Cell(String::from("w"), 0), Cell(String::from("b"), 1)),
        // 2*v = 0 in c
        (Cell(String::from("a"), 1), Cell(String::from("U"), 2)),
        (Cell(String::from("b"), 1), Cell(String::from("U"), 2)),
        (Cell(String::from("c"), 1), Cell(String::from("U"), 2)),
        (Cell(String::from("a"), 1), Cell(String::from("L"), 2)),
        (Cell(String::from("b"), 1), Cell(String::from("L"), 2)),
        (Cell(String::from("c"), 1), Cell(String::from("L"), 2)),
    ];

    Complex::from_face_relations(face_relations)

}

pub fn triangle() -> Complex {
    let face_relations = [
        (Cell(String::from("a"), 0), Cell(String::from("ab"), 1)),
        (Cell(String::from("b"), 0), Cell(String::from("ab"), 1)),
        (Cell(String::from("a"), 0), Cell(String::from("ac"), 1)),
        (Cell(String::from("c"), 0), Cell(String::from("ac"), 1)),
        (Cell(String::from("b"), 0), Cell(String::from("bc"), 1)),
        (Cell(String::from("c"), 0), Cell(String::from("bc"), 1)),
        (Cell(String::from("ab"), 1), Cell(String::from("abc"), 2)),
        (Cell(String::from("ac"), 1), Cell(String::from("abc"), 2)),
        (Cell(String::from("bc"), 1), Cell(String::from("abc"), 2)),
    ];

    Complex::from_face_relations(face_relations)
}

pub fn glued_polygon(n: usize) -> Complex {
    let face_relations = [
        (Cell(String::from("a"), 0), Cell(String::from("ab"), 1)),
        (Cell(String::from("b"), 0), Cell(String::from("ab"), 1)),
        (Cell(String::from("a"), 0), Cell(String::from("ba"), 1)),
        (Cell(String::from("b"), 0), Cell(String::from("ba"), 1)),
    ]
    .into_iter()
    .chain(
        (0..n)
            .map(|k| k * 2)
            .map(|k| (Cell(String::from("a"), 0), Cell(format!("e{k}"), 1))),
    )
    .chain(
        (0..n)
            .map(|k| k * 2 + 1)
            .map(|k| (Cell(String::from("b"), 0), Cell(format!("e{k}"), 1))),
    )
    .chain((0..2 * n).map(|k| (Cell(String::from("c"), 0), Cell(format!("e{k}"), 1))))
    .chain((0..2 * n).map(|k| (Cell(format!("e{k}"), 1), Cell(format!("t{k}"), 2))))
    .chain((0..2 * n).map(|k| {
        (
            Cell(format!("e{}", (k + 1) % (2 * n)), 1),
            Cell(format!("t{k}"), 2),
        )
    }))
    .collect::<Vec<_>>();

    Complex::from_face_relations(face_relations)
}

// since we do not have memory
pub fn generalized_dunce_hat_irregular(n: usize) -> Complex {
    let mut face_relations: Vec<(Cell<String>, Cell<String>)> = Vec::new();
    // vec![(Cell(String::from("a"), 0), Cell(String::from("aa"), 1))]; this is 2 % 2 = 0

    // for each edge of the polygon
    for i in 0..n {
        // connect "inner edge" with "boundary" vertex
        face_relations.push((Cell(String::from("a"), 0), Cell(format!("e{i}"), 1)));

        //connect "inner edge" with the center
        face_relations.push((Cell(String::from("c"), 0), Cell(format!("e{i}"), 1)));

        //connect "inner edges" with respective triangles. take into account that en=e0
        face_relations.push((Cell(format!("e{i}"), 1), Cell(format!("t{i}"), 2)));
        face_relations.push((
            Cell(format!("e{}", (i + 1) % n), 1),
            Cell(format!("t{i}"), 2),
        ));

        //connect "outer edge" with the triangle
        face_relations.push((Cell(String::from("aa"), 1), Cell(format!("t{i}"), 2)));
    }

    Complex::from_face_relations(face_relations)
}

pub fn generalized_dunce_hat(n: usize) -> Complex {
    // assert_eq!(n % 2, 1, "The number of edges of the polygon must be odd");

    // the outer edges of the polygon
    let mut face_relations: Vec<(Cell<String>, Cell<String>)> = vec![
        (Cell(String::from("a"), 0), Cell(String::from("ab"), 1)),
        (Cell(String::from("b"), 0), Cell(String::from("ab"), 1)),
        (Cell(String::from("a"), 0), Cell(String::from("ba"), 1)),
        (Cell(String::from("b"), 0), Cell(String::from("ba"), 1)),
    ];

    // for each edge of the polygon
    for i in 0..n {
        let i_left = 2 * i;
        let i_right = 2 * i + 1;

        // connect "inner edges" with "boundary" vertices
        face_relations.push((Cell(String::from("a"), 0), Cell(format!("e{i_left}"), 1)));
        face_relations.push((Cell(String::from("b"), 0), Cell(format!("e{i_right}"), 1)));

        //connect "inner edges" with the center
        face_relations.push((Cell(String::from("c"), 0), Cell(format!("e{i_left}"), 1)));
        face_relations.push((Cell(String::from("c"), 0), Cell(format!("e{i_right}"), 1)));

        //connect "inner edges" with respective triangles. take into account that en=e0, which can
        //only happen for the right side
        face_relations.push((Cell(format!("e{i_left}"), 1), Cell(format!("t{i_left}"), 2)));
        face_relations.push((
            Cell(format!("e{}", i_left + 1), 1),
            Cell(format!("t{i_left}"), 2),
        ));
        face_relations.push((
            Cell(format!("e{i_right}"), 1),
            Cell(format!("t{i_right}"), 2),
        ));
        face_relations.push((
            Cell(format!("e{}", (i_right + 1) % (2 * n)), 1),
            Cell(format!("t{i_right}"), 2),
        ));

        // if i is even, we connect ab to t_left and ba to t_right, otherwise other way around
        if i % 2 == 0 {
            face_relations.push((Cell(String::from("ab"), 1), Cell(format!("t{i_left}"), 2)));
            face_relations.push((Cell(String::from("ba"), 1), Cell(format!("t{i_right}"), 2)));
        } else {
            face_relations.push((Cell(String::from("ba"), 1), Cell(format!("t{i_left}"), 2)));
            face_relations.push((Cell(String::from("ab"), 1), Cell(format!("t{i_right}"), 2)));
        }
    }

    Complex::from_face_relations(face_relations)
}
