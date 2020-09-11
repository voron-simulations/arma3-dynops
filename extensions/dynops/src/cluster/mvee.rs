use crate::cluster::{Area, Coordinate};
use nalgebra::{DMatrix, DVector};
use std::f64::consts::*;

// Sourced from https://stackoverflow.com/a/1768440/1732138
pub fn get_mvee(coords: &[Coordinate], tolerance: f64) -> Area {
    let d = 2;
    let len = coords.len();
    let q: DMatrix<f64> = DMatrix::from_fn(d + 1, len, |r, c| match r {
        0 => coords[c][0],
        1 => coords[c][1],
        _ => 1.,
    });
    let q_t = q.transpose();

    let mut u = DVector::from_element(len, 1. / (len as f64));

    loop {
        let weighted_coords = &q * DMatrix::from_diagonal(&u) * &q_t;
        let deviations = (&q_t * weighted_coords.try_inverse().unwrap() * &q).diagonal();
        let (max_i, max_v) = deviations.argmax();
        let df = (d + 1) as f64;
        let step_size = (max_v - df) / df / (max_v - 1.);
        let mut new_u = &u * (1. - step_size);
        new_u[max_i] = new_u[max_i] + step_size;

        let err = (&new_u - &u).norm();
        u = new_u;
        if err < tolerance {
            break;
        }
    }

    let p = q.rows(0, d);
    let pu = &p * &u;
    let pt = &p.transpose();
    let put = &pu.transpose();

    // Resulting ellipse A-matrix
    let a_matrix = (p * DMatrix::from_diagonal(&u) * pt - pu * put) * (d as f64);
    // Center
    let c = p * u;

    let svd = a_matrix.svd(true, false);
    let u = svd.u.unwrap();
    //println!("SVD: {} {}", &svd.singular_values, u);

    let a01 = u.row(0)[0];
    let a11 = u.row(1)[0];
    let angle = a11.atan2(a01);

    Area {
        center: Coordinate { x: c[0], y: c[1] },
        a: svd.singular_values[0].sqrt(),
        b: svd.singular_values[1].sqrt(),
        angle: (angle * 180. / PI),
    }
}

#[cfg(test)]
mod tests {
    use crate::cluster::get_mvee;
    use crate::cluster::{Area, Coordinate};
    use std::f64::consts::*;

    // #[test]
    // fn mvee_one_point() {
    //     let input = vec![Coordinate { x: 0., y: 0. }];
    //     assert_eq!(
    //         Area {
    //             center: Coordinate { x: 0., y: 0. },
    //             a: 0.,
    //             b: 0.,
    //             angle: 0.
    //         },
    //         get_mvee(&input, 0.1)
    //     );
    // }

    // #[test]
    // fn mvee_two_points() {
    //     let input = vec![Coordinate { x: -1., y: 0. }, Coordinate { x: 1., y: 0. }];
    //     assert_eq!(
    //         Area {
    //             center: Coordinate { x: 0., y: 0. },
    //             a: 1.,
    //             b: 0.,
    //             angle: 0.
    //         },
    //         get_mvee(&input, 0.1)
    //     );
    // }

    #[test]
    fn mvee_four_points_circle_center() {
        let input = vec![
            Coordinate { x: -1., y: -1. },
            Coordinate { x: -1., y: 1. },
            Coordinate { x: 1., y: -1. },
            Coordinate { x: 1., y: 1. },
        ];
        assert_eq!(
            Area {
                center: Coordinate { x: 0., y: 0. },
                a: SQRT_2,
                b: SQRT_2,
                angle: 0.0
            },
            get_mvee(&input, 0.1)
        );
    }

    #[test]
    fn mvee_four_points_circle_shifted() {
        let input = vec![
            Coordinate { x: 4., y: 7. },
            Coordinate { x: 6., y: 9. },
            Coordinate { x: 4., y: 9. },
            Coordinate { x: 6., y: 7. },
        ];
        assert_eq!(
            Area {
                center: Coordinate { x: 5., y: 8. },
                a: SQRT_2,
                b: SQRT_2,
                angle: 0.0
            },
            get_mvee(&input, 0.1)
        );
    }

    #[test]
    fn mvee_four_points_circle_3() {
        let input = vec![
            Coordinate { x: -3., y: -3. },
            Coordinate { x: -3., y: 3. },
            Coordinate { x: 3., y: -3. },
            Coordinate { x: 3., y: 3. },
        ];
        assert_eq!(
            Area {
                center: Coordinate { x: 0., y: 0. },
                a: 3. * SQRT_2,
                b: 3. * SQRT_2,
                angle: 0.0
            },
            get_mvee(&input, 0.1)
        );
    }

    #[test]
    fn mvee_four_points_horizontal() {
        let input = vec![
            Coordinate { x: -2., y: -1. },
            Coordinate { x: -2., y: 1. },
            Coordinate { x: 2., y: -1. },
            Coordinate { x: 2., y: 1. },
        ];
        assert_eq!(
            Area {
                center: Coordinate { x: 0., y: 0. },
                a: 2. * SQRT_2,
                b: SQRT_2,
                angle: 0.0
            },
            get_mvee(&input, 0.1)
        );
    }

    #[test]
    fn mvee_four_points_vertical() {
        let input = vec![
            Coordinate { x: -1., y: -2. },
            Coordinate { x: -1., y: 2. },
            Coordinate { x: 1., y: -2. },
            Coordinate { x: 1., y: 2. },
        ];
        assert_eq!(
            Area {
                center: Coordinate { x: 0., y: 0. },
                a: SQRT_2,
                b: 2. * SQRT_2,
                angle: 0.0
            },
            get_mvee(&input, 0.1)
        );
    }

    #[test]
    fn mvee_four_points_diagonal1() {
        let input = vec![
            Coordinate { x: 0., y: 5. },
            Coordinate { x: 1., y: 6. },
            Coordinate { x: 5., y: 0. },
            Coordinate { x: 6., y: 1. },
        ];
        assert_eq!(
            Area {
                center: Coordinate { x: 3., y: 3. },
                a: 1.,
                b: 5.,
                angle: 45.0
            },
            get_mvee(&input, 0.1)
        );
    }

    #[test]
    fn mvee_four_points_diagonal2() {
        let input = vec![
            Coordinate { x: 1., y: 0. },
            Coordinate { x: 0., y: 1. },
            Coordinate { x: 5., y: 6. },
            Coordinate { x: 6., y: 5. },
        ];
        assert_eq!(
            Area {
                center: Coordinate { x: 3., y: 3. },
                a: 1.,
                b: 5.,
                angle: -45.0
            },
            get_mvee(&input, 0.1)
        );
    }
}
