use crate::shape::{Ellipse, Rectangle};
use nalgebra::{DMatrix, DVector, Vector2};
// Sourced from https://stackoverflow.com/a/1768440/1732138
pub fn bounding_ellipse(coords: &[Vector2<f64>], tolerance: f64) -> Ellipse {
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
        new_u[max_i] += step_size;
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
    Ellipse {
        x: c[0],
        y: c[1],
        a: svd.singular_values[0].sqrt(),
        b: svd.singular_values[1].sqrt(),
        r: angle,
    }
}
fn bounding_aa_rec(coords: &[Vector2<f64>]) -> Rectangle {
    let mut xmin: f64 = f64::MAX;
    let mut xmax: f64 = f64::MIN;
    let mut ymin: f64 = f64::MAX;
    let mut ymax: f64 = f64::MIN;
    for point in coords {
        xmin = xmin.min(point.x);
        xmax = xmax.max(point.x);
        ymin = ymin.min(point.y);
        ymax = ymax.max(point.y);
    }
    let x = (xmax + xmin) / 2.0;
    let y = (ymax + ymin) / 2.0;
    Rectangle {
        x,
        y,
        a: (xmax - xmin) / 2.0,
        b: (ymax - ymin) / 2.0,
        r: 0.,
    }
}
#[cfg(test)]
mod tests {
    use super::bounding_ellipse;
    use crate::shape::Ellipse;
    use nalgebra::Vector2;
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, SQRT_2};

    fn distance(a1: &Ellipse, a2: &Ellipse) -> f64 {
        let d = f64::powf(a1.x - a2.x, 2.)
            + f64::powf(a1.y - a2.y, 2.)
            + f64::powf(a1.a - a2.a, 2.)
            + f64::powf(a1.b - a2.b, 2.)
            + f64::powf(a1.r - a2.r, 2.);
        d.sqrt()
    }

    fn run_mvee_test(points: &[Vector2<f64>], tolerance: f64, expected: Ellipse) {
        let actual = bounding_ellipse(points, tolerance);
        for point in points {
            assert!(
                expected.contains_vec(point),
                "{}", "Precondition failed: input point {point} must be in expected area: {expected}"
            )
        }
        assert!(
            distance(&expected, &actual) < 1e-3,
            "{}", "Expected: {expected}, got: {actual}"
        );
        let extended_actual = Ellipse {a: actual.a+tolerance, b: actual.b+tolerance, ..actual};
        for point in points {
            assert!(
                extended_actual.contains_vec(point),
                "{}", "Precondition failed: input point {point} must be in expected area: {actual}"
            )
        }
    }
    #[test]
    fn mvee_one_point() {
        let input = vec![Vector2::new(0., 0.)];
        // Just verify it doesn't panic
        bounding_ellipse(&input, 0.1);
    }
    #[test]
    fn mvee_two_points() {
        let input = vec![Vector2::new(-1., 0.), Vector2::new(1., 0.)];
        // Just verify it doesn't panic
        bounding_ellipse(&input, 0.1);
    }
    #[test]
    fn mvee_four_points_circle_center() {
        let input = vec![
            Vector2::new(-1., -1.),
            Vector2::new(-1., 1.),
            Vector2::new(1., -1.),
            Vector2::new(1., 1.),
        ];
        run_mvee_test(
            &input,
            0.1,
            Ellipse {
                x: 0.,
                y: 0.,
                a: SQRT_2,
                b: SQRT_2,
                r: 0.,
            },
        );
    }
    #[test]
    fn mvee_four_points_circle_shifted() {
        let input = vec![
            Vector2::new(4., 7.),
            Vector2::new(6., 9.),
            Vector2::new(4., 9.),
            Vector2::new(6., 7.),
        ];
        run_mvee_test(
            &input,
            0.1,
            Ellipse {
                x: 5.,
                y: 8.,
                a: SQRT_2,
                b: SQRT_2,
                r: 0.,
            },
        );
    }
    #[test]
    fn mvee_four_points_circle_3() {
        let input = vec![
            Vector2::new(-3., -3.),
            Vector2::new(-3., 3.),
            Vector2::new(3., -3.),
            Vector2::new(3., 3.),
        ];
        run_mvee_test(
            &input,
            0.1,
            Ellipse {
                x: 0.,
                y: 0.,
                a: 3. * SQRT_2,
                b: 3. * SQRT_2,
                r: 0.,
            },
        );
    }
    #[test]
    fn mvee_four_points_horizontal() {
        let input = vec![
            Vector2::new(-2., -1.),
            Vector2::new(-2., 1.),
            Vector2::new(2., -1.),
            Vector2::new(2., 1.),
        ];
        run_mvee_test(
            &input,
            0.1,
            Ellipse {
                x: 0.,
                y: 0.,
                a: 2. * SQRT_2,
                b: SQRT_2,
                r: 0.,
            },
        );
    }
    #[test]
    fn mvee_four_points_vertical() {
        let input = vec![
            Vector2::new(-1., -2.),
            Vector2::new(-1., 2.),
            Vector2::new(1., -2.),
            Vector2::new(1., 2.),
        ];
        run_mvee_test(
            &input,
            0.1,
            Ellipse {
                x: 0.,
                y: 0.,
                a: 2. * SQRT_2,
                b: SQRT_2,
                r: FRAC_PI_2,
            },
        );
    }
    #[test]
    fn mvee_four_points_diagonal1() {
        let input = vec![
            Vector2::new(0., 5.),
            Vector2::new(1., 6.),
            Vector2::new(5., 0.),
            Vector2::new(6., 1.),
        ];
        run_mvee_test(
            &input,
            0.1,
            Ellipse {
                x: 3.,
                y: 3.,
                a: 5.,
                b: 1.,
                r: -FRAC_PI_4,
            },
        );
    }
    #[test]
    fn mvee_four_points_diagonal2() {
        let input = vec![
            Vector2::new(1., 0.),
            Vector2::new(0., 1.),
            Vector2::new(5., 6.),
            Vector2::new(6., 5.),
        ];
        run_mvee_test(
            &input,
            0.1,
            Ellipse {
                x: 3.,
                y: 3.,
                a: 5.,
                b: 1.,
                r: FRAC_PI_4,
            },
        );
    }
}
