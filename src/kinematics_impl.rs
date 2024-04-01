use std::f64::{consts::PI};
use crate::kinematic_traits::kinematics_traits::{Kinematics, Solutions, Pose};
use crate::parameters::opw_kinematics::Parameters;
use nalgebra::{Isometry3, Matrix3, OVector, Rotation3, Translation3, U3, Unit, UnitQuaternion,
               Vector3, SMatrix};

pub(crate) struct OPWKinematics {
    parameters: Parameters,
    unit_z: Unit<OVector<f64, U3>>,
}

impl OPWKinematics {
    /// Creates a new `OPWKinematics` instance with the given parameters.
    pub fn new(parameters: Parameters) -> Self {
        OPWKinematics {
            parameters,
            unit_z: Unit::new_normalize(Vector3::z_axis().into_inner()),
        }
    }
}

// Compare two poses with the given tolerance.
fn compare_poses(ta: &Isometry3<f64>, tb: &Isometry3<f64>, tolerance: f64) -> bool {
    let translation_distance = (ta.translation.vector - tb.translation.vector).norm();
    let angular_distance = ta.rotation.angle_to(&tb.rotation);

    if translation_distance.abs() > tolerance {
        println!("Translation Error: {}", translation_distance);
        return false;
    }

    if angular_distance.abs() > tolerance {
        println!("Angular Error: {}", angular_distance);
        return false;
    }
    true
}


impl Kinematics for OPWKinematics {
    fn inverse(&self, pose: &Pose) -> Solutions {
        let params = &self.parameters;

        let mut solutions: Solutions = Solutions::from_element(f64::NAN);

        // Adjust to wrist center
        let matrix = pose.rotation.to_rotation_matrix();
        let translation_vector = pose.translation.vector; // Get the translation vector component
        let scaled_z_axis = params.c4 * matrix.transform_vector(&Vector3::z_axis()); // Scale and rotate the z-axis vector

        let c = translation_vector - scaled_z_axis;

        let nx1 = ((c.x * c.x + c.y * c.y) - params.b * params.b).sqrt() - params.a1;

        let tmp1 = c.y.atan2(c.x); // Rust's method call syntax for atan2(y, x)
        let tmp2 = params.b.atan2(nx1 + params.a1);

        let theta1_i = tmp1 - tmp2;
        let theta1_ii = tmp1 + tmp2 - PI;

        let tmp3 = c.z - params.c1; // Access z directly for nalgebra's Vector3
        let s1_2 = nx1 * nx1 + tmp3 * tmp3;

        let tmp4 = nx1 + 2.0 * params.a1;
        let s2_2 = tmp4 * tmp4 + tmp3 * tmp3;
        let kappa_2 = params.a2 * params.a2 + params.c3 * params.c3;

        let c2_2 = params.c2 * params.c2;

        let tmp5 = s1_2 + c2_2 - kappa_2;

        let s1 = f64::sqrt(s1_2);
        let s2 = f64::sqrt(s2_2);

        let tmp13 = f64::acos(tmp5 / (2.0 * s1 * params.c2));
        let tmp14 = f64::atan2(nx1, c.z - params.c1);
        let theta2_i = -tmp13 + tmp14;
        let theta2_ii = tmp13 + tmp14;

        let tmp6 = s2_2 + c2_2 - kappa_2;

        let tmp15 = f64::acos(tmp6 / (2.0 * s2 * params.c2));
        let tmp16 = f64::atan2(nx1 + 2.0 * params.a1, c.z - params.c1);
        let theta2_iii = -tmp15 - tmp16;
        let theta2_iv = tmp15 - tmp16;

        // theta3
        let tmp7 = s1_2 - c2_2 - kappa_2;
        let tmp8 = s2_2 - c2_2 - kappa_2;
        let tmp9 = 2.0 * params.c2 * f64::sqrt(kappa_2); // Using f64::sqrt for the square root calculation
        let tmp10 = f64::atan2(params.a2, params.c3); // atan2 used directly on f64 values

        let tmp11 = f64::acos(tmp7 / tmp9); // acos used directly on the f64 value
        let theta3_i = tmp11 - tmp10;
        let theta3_ii = -tmp11 - tmp10;

        let tmp12 = f64::acos(tmp8 / tmp9); // acos used directly on the f64 value
        let theta3_iii = tmp12 - tmp10;
        let theta3_iv = -tmp12 - tmp10;

        let theta1_i_sin = theta1_i.sin();
        let theta1_i_cos = theta1_i.cos();
        let theta1_ii_sin = theta1_ii.sin();
        let theta1_ii_cos = theta1_ii.cos();

        // orientation part
        let sin1: [f64; 4] = [
            theta1_i_sin, theta1_i_sin, theta1_ii_sin, theta1_ii_sin,
        ];

        let cos1: [f64; 4] = [
            theta1_i_cos, theta1_i_cos, theta1_ii_cos, theta1_ii_cos
        ];

        let s23: [f64; 4] = [
            (theta2_i + theta3_i).sin(),
            (theta2_ii + theta3_ii).sin(),
            (theta2_iii + theta3_iii).sin(),
            (theta2_iv + theta3_iv).sin(),
        ];

        let c23: [f64; 4] = [
            (theta2_i + theta3_i).cos(),
            (theta2_ii + theta3_ii).cos(),
            (theta2_iii + theta3_iii).cos(),
            (theta2_iv + theta3_iv).cos(),
        ];

        let m: [f64; 4] = [
            matrix[(0, 2)] * s23[0] * cos1[0] + matrix[(1, 2)] * s23[0] * sin1[0] + matrix[(2, 2)] * c23[0],
            matrix[(0, 2)] * s23[1] * cos1[1] + matrix[(1, 2)] * s23[1] * sin1[1] + matrix[(2, 2)] * c23[1],
            matrix[(0, 2)] * s23[2] * cos1[2] + matrix[(1, 2)] * s23[2] * sin1[2] + matrix[(2, 2)] * c23[2],
            matrix[(0, 2)] * s23[3] * cos1[3] + matrix[(1, 2)] * s23[3] * sin1[3] + matrix[(2, 2)] * c23[3],
        ];

        let theta5_i = f64::atan2((1.0 - m[0] * m[0]).sqrt(), m[0]);
        let theta5_ii = f64::atan2((1.0 - m[1] * m[1]).sqrt(), m[1]);
        let theta5_iii = f64::atan2((1.0 - m[2] * m[2]).sqrt(), m[2]);
        let theta5_iv = f64::atan2((1.0 - m[3] * m[3]).sqrt(), m[3]);

        let theta5_v = -theta5_i;
        let theta5_vi = -theta5_ii;
        let theta5_vii = -theta5_iii;
        let theta5_viii = -theta5_iv;

        let zero_threshold: f64 = 1e-6;
        let theta4_i;
        let theta6_i;

        if theta5_i.abs() < zero_threshold {
            theta4_i = 0.0;
            let xe = Vector3::new(matrix[(0, 0)], matrix[(1, 0)], matrix[(2, 0)]);
            let mut rc = Matrix3::zeros(); // Assuming Matrix3::zeros() creates a 3x3 matrix filled with 0.0

            // Set columns of Rc
            rc.set_column(1, &Vector3::new(-theta1_i.sin(), theta1_i.cos(), 0.0)); // yc
            rc.set_column(2, &Vector3::new(matrix[(0, 2)], matrix[(1, 2)], matrix[(2, 2)])); // zc = ze
            rc.set_column(0, &rc.column(1).cross(&rc.column(2))); // xc

            let xec = rc.transpose() * xe;
            theta6_i = xec[1].atan2(xec[0]);
        } else {
            let theta4_iy = matrix[(1, 2)] * cos1[0] - matrix[(0, 2)] * sin1[0];
            let theta4_ix = matrix[(0, 2)] * c23[0] * cos1[0] + matrix[(1, 2)] * c23[0] * sin1[0] - matrix[(2, 2)] * s23[0];
            theta4_i = theta4_iy.atan2(theta4_ix);

            let theta6_iy = matrix[(0, 1)] * s23[0] * cos1[0] + matrix[(1, 1)] * s23[0] * sin1[0] + matrix[(2, 1)] * c23[0];
            let theta6_ix = -matrix[(0, 0)] * s23[0] * cos1[0] - matrix[(1, 0)] * s23[0] * sin1[0] - matrix[(2, 0)] * c23[0];
            theta6_i = theta6_iy.atan2(theta6_ix);
        }

        let theta4_ii;
        let theta6_ii;

        if theta5_ii.abs() < zero_threshold {
            theta4_ii = 0.0;
            let xe = Vector3::new(matrix[(0, 0)], matrix[(1, 0)], matrix[(2, 0)]);
            let mut rc = Matrix3::zeros();

            // Set columns of Rc
            rc.set_column(1, &Vector3::new(-theta1_i.sin(), theta1_i.cos(), 0.0)); // yc
            rc.set_column(2, &Vector3::new(matrix[(0, 2)], matrix[(1, 2)], matrix[(2, 2)])); // zc = ze
            rc.set_column(0, &rc.column(1).cross(&rc.column(2))); // xc

            let xec = rc.transpose() * xe;
            theta6_ii = xec[1].atan2(xec[0]);
        } else {
            let theta4_iiy = matrix[(1, 2)] * cos1[1] - matrix[(0, 2)] * sin1[1];
            let theta4_iix = matrix[(0, 2)] * c23[1] * cos1[1] + matrix[(1, 2)] * c23[1] * sin1[1] - matrix[(2, 2)] * s23[1];
            theta4_ii = theta4_iiy.atan2(theta4_iix);

            let theta6_iiy = matrix[(0, 1)] * s23[1] * cos1[1] + matrix[(1, 1)] * s23[1] * sin1[1] + matrix[(2, 1)] * c23[1];
            let theta6_iix = -matrix[(0, 0)] * s23[1] * cos1[1] - matrix[(1, 0)] * s23[1] * sin1[1] - matrix[(2, 0)] * c23[1];
            theta6_ii = theta6_iiy.atan2(theta6_iix);
        }

        let theta4_iii;
        let theta6_iii;

        if theta5_iii.abs() < zero_threshold {
            theta4_iii = 0.0;
            let xe = Vector3::new(matrix[(0, 0)], matrix[(1, 0)], matrix[(2, 0)]);
            let mut rc = Matrix3::zeros();

            // Set columns of Rc
            rc.set_column(1, &Vector3::new(-theta1_ii.sin(), theta1_ii.cos(), 0.0)); // yc
            rc.set_column(2, &Vector3::new(matrix[(0, 2)], matrix[(1, 2)], matrix[(2, 2)])); // zc = ze
            rc.set_column(0, &rc.column(1).cross(&rc.column(2))); // xc

            let xec = rc.transpose() * xe;
            theta6_iii = xec[1].atan2(xec[0]);
        } else {
            let theta4_iiiy = matrix[(1, 2)] * cos1[2] - matrix[(0, 2)] * sin1[2];
            let theta4_iiix = matrix[(0, 2)] * c23[2] * cos1[2] + matrix[(1, 2)] * c23[2] * sin1[2] - matrix[(2, 2)] * s23[2];
            theta4_iii = theta4_iiiy.atan2(theta4_iiix);

            let theta6_iiiy = matrix[(0, 1)] * s23[2] * cos1[2] + matrix[(1, 1)] * s23[2] * sin1[2] + matrix[(2, 1)] * c23[2];
            let theta6_iiix = -matrix[(0, 0)] * s23[2] * cos1[2] - matrix[(1, 0)] * s23[2] * sin1[2] - matrix[(2, 0)] * c23[2];
            theta6_iii = theta6_iiiy.atan2(theta6_iiix);
        }

        let theta4_iv;
        let theta6_iv;

        if theta5_iv.abs() < zero_threshold {
            theta4_iv = 0.0;
            let xe = Vector3::new(matrix[(0, 0)], matrix[(1, 0)], matrix[(2, 0)]);
            let mut rc = Matrix3::zeros();
            rc.set_column(1, &Vector3::new(-theta1_ii.sin(), theta1_ii.cos(), 0.0));
            rc.set_column(2, &Vector3::new(matrix[(0, 2)], matrix[(1, 2)], matrix[(2, 2)]));
            rc.set_column(0, &rc.column(1).cross(&rc.column(2)));

            let xec = rc.transpose() * xe;
            theta6_iv = xec[1].atan2(xec[0]);
        } else {
            let theta4_ivy = matrix[(1, 2)] * cos1[3] - matrix[(0, 2)] * sin1[3];
            let theta4_ivx = matrix[(0, 2)] * c23[3] * cos1[3] + matrix[(1, 2)] * c23[3] * sin1[3] - matrix[(2, 2)] * s23[3];
            theta4_iv = theta4_ivy.atan2(theta4_ivx);

            let theta6_ivy = matrix[(0, 1)] * s23[3] * cos1[3] + matrix[(1, 1)] * s23[3] * sin1[3] + matrix[(2, 1)] * c23[3];
            let theta6_ivx = -matrix[(0, 0)] * s23[3] * cos1[3] - matrix[(1, 0)] * s23[3] * sin1[3] - matrix[(2, 0)] * c23[3];
            theta6_iv = theta6_ivy.atan2(theta6_ivx);
        }

        let theta4_v = theta4_i + PI;
        let theta4_vi = theta4_ii + PI;
        let theta4_vii = theta4_iii + PI;
        let theta4_viii = theta4_iv + PI;

        let theta6_v = theta6_i - PI;
        let theta6_vi = theta6_ii - PI;
        let theta6_vii = theta6_iii - PI;
        let theta6_viii = theta6_iv - PI;


        let offsets = &params.offsets;
        let signs: Vec<f64> = params.sign_corrections.iter().map(|&x| x as f64).collect();

        let theta = SMatrix::<f64, 6, 8>::from_columns(&[
            SMatrix::<f64, 6, 1>::new(theta1_i, theta2_i, theta3_i, theta4_i, theta5_i, theta6_i),
            SMatrix::<f64, 6, 1>::new(theta1_i, theta2_ii, theta3_ii, theta4_ii, theta5_ii, theta6_ii),
            SMatrix::<f64, 6, 1>::new(theta1_ii, theta2_iii, theta3_iii, theta4_iii, theta5_iii, theta6_iii),
            SMatrix::<f64, 6, 1>::new(theta1_ii, theta2_iv, theta3_iv, theta4_iv, theta5_iv, theta6_iv),
            SMatrix::<f64, 6, 1>::new(theta1_i, theta2_i, theta3_i, theta4_v, theta5_v, theta6_v),
            SMatrix::<f64, 6, 1>::new(theta1_i, theta2_ii, theta3_ii, theta4_vi, theta5_vi, theta6_vi),
            SMatrix::<f64, 6, 1>::new(theta1_ii, theta2_iii, theta3_iii, theta4_vii, theta5_vii, theta6_vii),
            SMatrix::<f64, 6, 1>::new(theta1_ii, theta2_iv, theta3_iv, theta4_viii, theta5_viii, theta6_viii),
        ]);

        let offsets_matrix = SMatrix::<f64, 6, 1>::from_column_slice(offsets);
        let signs_matrix = SMatrix::<f64, 6, 1>::from_column_slice(&signs);


        for i in 0..solutions.ncols() {
            for j in 0..solutions.nrows() {
                // Directly accessing and modifying elements in the matrix
                solutions[(j, i)] = (theta[(j, i)] + offsets_matrix[j]) * signs_matrix[j];
            }
        }

        // Debug check. Solution failing cross-verification is flagged
        // as invalid. This loop also normalizes valid solutions to 0
        for si in 0..solutions.ncols() {
            let mut valid = true;
            for ji in 0..6 {
                let mut angle = solutions[(ji, si)];
                if angle.is_finite() {
                    while angle > PI {
                        angle -= 2.0 * PI;
                    }
                    while angle < -PI {
                        angle += 2.0 * PI;
                    }
                    solutions[(ji, si)] = angle;
                } else {
                    valid = false;
                    break;
                }
            };
            if valid {
                let column = solutions.column(si);
                let solution: [f64; 6] = [
                    column[(0, 0)],
                    column[(1, 0)],
                    column[(2, 0)],
                    column[(3, 0)],
                    column[(4, 0)],
                    column[(5, 0)],
                ];
                let check_pose = self.forward(&solution);
                if !compare_poses(&pose, &check_pose, 1e-3) {
                    println!("********** Pose Failure *********");
                    // Kill the entry making the failed solution invalid.
                    solutions[(0, si)] = f64::NAN;
                }
            }
        }

        solutions
    }

    fn forward(&self, joints: &[f64; 6]) -> Pose {
        let p = &self.parameters;

        let q: Vec<f64> = joints.iter()
            .zip(p.sign_corrections.iter())
            .zip(p.offsets.iter())
            .map(|((&joint, &sign_correction), &offset)| {
                joint * sign_correction as f64 - offset
            })
            .collect();

        let psi3 = f64::atan2(p.a2, p.c3);
        let k = f64::sqrt(p.a2 * p.a2 + p.c3 * p.c3);

        let cx1 = p.c2 * f64::sin(q[1]) + k * f64::sin(q[1] + q[2] + psi3) + p.a1;
        let cy1 = p.b;
        let cz1 = p.c2 * f64::cos(q[1]) + k * f64::cos(q[1] + q[2] + psi3);

        let cx0 = cx1 * f64::cos(q[0]) - cy1 * f64::sin(q[0]);
        let cy0 = cx1 * f64::sin(q[0]) + cy1 * f64::cos(q[0]);
        let cz0 = cz1 + p.c1;

        let s1 = f64::sin(q[0]);
        let s2 = f64::sin(q[1]);
        let s3 = f64::sin(q[2]);
        let s4 = f64::sin(q[3]);
        let s5 = f64::sin(q[4]);
        let s6 = f64::sin(q[5]);

        let c1 = f64::cos(q[0]);
        let c2 = f64::cos(q[1]);
        let c3 = f64::cos(q[2]);
        let c4 = f64::cos(q[3]);
        let c5 = f64::cos(q[4]);
        let c6 = f64::cos(q[5]);

        let r_0c = Matrix3::new(
            c1 * c2 * c3 - c1 * s2 * s3, -s1, c1 * c2 * s3 + c1 * s2 * c3,
            s1 * c2 * c3 - s1 * s2 * s3, c1, s1 * c2 * s3 + s1 * s2 * c3,
            -s2 * c3 - c2 * s3, 0.0, -s2 * s3 + c2 * c3,
        );

        let r_ce = Matrix3::new(
            c4 * c5 * c6 - s4 * s6, -c4 * c5 * s6 - s4 * c6, c4 * s5,
            s4 * c5 * c6 + c4 * s6, -s4 * c5 * s6 + c4 * c6, s4 * s5,
            -s5 * c6, s5 * s6, c5,
        );

        let r_oe = r_0c * r_ce;

        let translation = Vector3::new(cx0, cy0, cz0) + p.c4 * r_oe * *self.unit_z;
        let rotation = Rotation3::from_matrix_unchecked(r_oe);

        Pose::from_parts(Translation3::from(translation),
                         UnitQuaternion::from_rotation_matrix(&rotation))
    }
}
