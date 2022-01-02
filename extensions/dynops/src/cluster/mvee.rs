use crate::types::{Area, AreaKind, Position2d};
use nalgebra::{DMatrix, DVector};
use std::f64::consts::*;

// Sourced from https://stackoverflow.com/a/1768440/1732138
pub fn get_mvee(coords: &[Position2d], tolerance: f64) -> Area {
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
        let inverse = weighted_coords.try_inverse();
        if inverse.is_none() {
            break;
        }
        let deviations = (&q_t * inverse.unwrap() * &q).diagonal();
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
        center: Position2d::new(c[0], c[1]),
        xsize: svd.singular_values[0].sqrt(),
        ysize: svd.singular_values[1].sqrt(),
        angle: angle,
        kind: AreaKind::Ellipse,
    }
}

#[cfg(test)]
mod tests {
    use crate::cluster::mvee::get_mvee;
    use crate::types::{distance, Area, AreaKind, Position2d};
    use std::f64::consts::{FRAC_PI_4, SQRT_2};

    fn run_mvee_test(points: &[Position2d], tolerance: f64, expected: Area) {
        let actual = get_mvee(&points, tolerance);

        for point in points {
            assert!(
                expected.contains_tolerance(point, 1e-3),
                "Precondition failed: input point must be in expected area: {}",
                point
            )
        }

        assert!(
            distance(expected, actual) < 1e-5,
            "Expected result: {}, got: {}",
            expected,
            actual
        );

        for point in points {
            assert!(
                actual.contains_tolerance(point, 1e-3),
                "Result check failed: input point must be in calculated area: {}",
                point
            )
        }
    }

    #[test]
    fn mvee_one_point() {
        let input = vec![Position2d::new(0., 0.)];
        // Just verify it doesn't panic
        get_mvee(&input, 0.1);
    }

    #[test]
    fn mvee_two_points() {
        let input = vec![Position2d::new(-1., 0.), Position2d::new(1., 0.)];
        // Just verify it doesn't panic
        get_mvee(&input, 0.1);
    }

    #[test]
    fn mvee_four_points_circle_center() {
        let input = vec![
            Position2d::new(-1., -1.),
            Position2d::new(-1., 1.),
            Position2d::new(1., -1.),
            Position2d::new(1., 1.),
        ];
        run_mvee_test(
            &input,
            0.1,
            Area {
                center: Position2d::new(0., 0.),
                xsize: SQRT_2,
                ysize: SQRT_2,
                kind: AreaKind::Ellipse,
                angle: 0.,
            },
        );
    }

    #[test]
    fn mvee_four_points_circle_shifted() {
        let input = vec![
            Position2d::new(4., 7.),
            Position2d::new(6., 9.),
            Position2d::new(4., 9.),
            Position2d::new(6., 7.),
        ];
        run_mvee_test(
            &input,
            0.1,
            Area {
                center: Position2d::new(5., 8.),
                xsize: SQRT_2,
                ysize: SQRT_2,
                kind: AreaKind::Ellipse,
                angle: 0.,
            },
        );
    }

    #[test]
    fn mvee_four_points_circle_3() {
        let input = vec![
            Position2d::new(-3., -3.),
            Position2d::new(-3., 3.),
            Position2d::new(3., -3.),
            Position2d::new(3., 3.),
        ];
        run_mvee_test(
            &input,
            0.1,
            Area {
                center: Position2d::new(0., 0.),
                xsize: 3. * SQRT_2,
                ysize: 3. * SQRT_2,
                kind: AreaKind::Ellipse,
                angle: 0.,
            },
        );
    }

    #[test]
    fn mvee_four_points_horizontal() {
        let input = vec![
            Position2d::new(-2., -1.),
            Position2d::new(-2., 1.),
            Position2d::new(2., -1.),
            Position2d::new(2., 1.),
        ];
        run_mvee_test(
            &input,
            0.1,
            Area {
                center: Position2d::new(0., 0.),
                xsize: 2. * SQRT_2,
                ysize: SQRT_2,
                kind: AreaKind::Ellipse,
                angle: 0.,
            },
        );
    }

    #[test]
    fn mvee_four_points_vertical() {
        let input = vec![
            Position2d::new(-1., -2.),
            Position2d::new(-1., 2.),
            Position2d::new(1., -2.),
            Position2d::new(1., 2.),
        ];
        run_mvee_test(
            &input,
            0.1,
            Area {
                center: Position2d::new(0., 0.),
                xsize: SQRT_2,
                ysize: 2. * SQRT_2,
                kind: AreaKind::Ellipse,
                angle: 0.,
            },
        );
    }

    #[test]
    fn mvee_four_points_diagonal1() {
        let input = vec![
            Position2d::new(0., 5.),
            Position2d::new(1., 6.),
            Position2d::new(5., 0.),
            Position2d::new(6., 1.),
        ];
        run_mvee_test(
            &input,
            0.1,
            Area {
                center: Position2d::new(3., 3.),
                xsize: 1.,
                ysize: 5.,
                kind: AreaKind::Ellipse,
                angle: FRAC_PI_4,
            },
        );
    }

    #[test]
    fn mvee_four_points_diagonal2() {
        let input = vec![
            Position2d::new(1., 0.),
            Position2d::new(0., 1.),
            Position2d::new(5., 6.),
            Position2d::new(6., 5.),
        ];
        run_mvee_test(
            &input,
            0.1,
            Area {
                center: Position2d::new(3., 3.),
                xsize: 1.,
                ysize: 5.,
                kind: AreaKind::Ellipse,
                angle: -FRAC_PI_4,
            },
        );
    }
}
