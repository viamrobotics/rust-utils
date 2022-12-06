use float_cmp::{ApproxEq, F64Margin};
use nalgebra::{Quaternion, Vector3, UnitQuaternion, UnitVector3};

const ANGLE_ACCEPTANCE: f64 = 0.0001;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EulerAngles {
    pub roll: f64,
    pub pitch: f64,
    pub yaw: f64
}

impl EulerAngles {
    pub fn new(roll: f64, pitch: f64, yaw: f64) -> Self {
        EulerAngles { roll, pitch, yaw }
    }

    /// Converts a quaternion into euler angles (in radians). The euler angles are 
    /// represented according to the Tait-Bryan formalism and applied 
    /// in the Z-Y'-X" order (where Z -> yaw, Y -> pitch, X -> roll).
    pub fn from_quaternion(quat: &Quaternion<f64>) -> Self {
        // get a normalized version of the quaternion
        let norm_quat = quat.normalize();

        // calculate yaw
        let yaw_sin_pitch_cos: f64 = 2.0 * ((norm_quat.w * norm_quat.k) + (norm_quat.i * norm_quat.j));
        let yaw_cos_pitch_cos: f64 = 1.0 - 2.0 * ((norm_quat.j * norm_quat.j) + (norm_quat.k * norm_quat.k));
        let yaw = yaw_sin_pitch_cos.atan2(yaw_cos_pitch_cos);

        // calculate pitch and roll
        let pitch_sin: f64 = 2.0 * ((norm_quat.w * norm_quat.j) - (norm_quat.k * norm_quat.i));
        let pitch: f64;
        let roll: f64;
        // for a pitch that is π / 2, we experience gimbal lock
        // and must calculate roll based on the real rotation and yaw
        if pitch_sin.abs() >= 1.0 {
            pitch = (std::f64::consts::PI / 2.0).copysign(pitch_sin);
            roll = (2.0 * norm_quat.i.atan2(norm_quat.w)) + yaw.copysign(pitch_sin);
        } else {
            pitch = pitch_sin.asin();
            let roll_sin_pitch_cos = 2.0 * ((norm_quat.w * norm_quat.i) + (norm_quat.j * norm_quat.k));
            let roll_cos_pitch_cos = 1.0 - 2.0 * ((norm_quat.i * norm_quat.i) + (norm_quat.j * norm_quat.j));
            roll = roll_sin_pitch_cos.atan2(roll_cos_pitch_cos);
        }

        EulerAngles { roll, pitch, yaw }
    }
}

impl From<Quaternion<f64>> for EulerAngles {
    fn from(quat: Quaternion<f64>) -> Self {
        // get a normalized version of the quaternion
        let norm_quat = quat.normalize();

        // calculate yaw
        let yaw_sin_pitch_cos: f64 = 2.0 * ((norm_quat.w * norm_quat.k) + (norm_quat.i * norm_quat.j));
        let yaw_cos_pitch_cos: f64 = 1.0 - 2.0 * ((norm_quat.j * norm_quat.j) + (norm_quat.k * norm_quat.k));
        let yaw = yaw_sin_pitch_cos.atan2(yaw_cos_pitch_cos);

        // calculate pitch and roll
        let pitch_sin: f64 = 2.0 * ((norm_quat.w * norm_quat.j) - (norm_quat.k * norm_quat.i));
        let pitch: f64;
        let roll: f64;
        // for a pitch that is π / 2, we experience gimbal lock
        // and must calculate roll based on the real rotation and yaw
        if pitch_sin.abs() >= 1.0 {
            pitch = (std::f64::consts::PI / 2.0).copysign(pitch_sin);
            roll = (2.0 * norm_quat.i.atan2(norm_quat.w)) + yaw.copysign(pitch_sin);
        } else {
            pitch = pitch_sin.asin();
            let roll_sin_pitch_cos = 2.0 * ((norm_quat.w * norm_quat.i) + (norm_quat.j * norm_quat.k));
            let roll_cos_pitch_cos = 1.0 - 2.0 * ((norm_quat.i * norm_quat.i) + (norm_quat.j * norm_quat.j));
            roll = roll_sin_pitch_cos.atan2(roll_cos_pitch_cos);
        }

        Self { roll, pitch, yaw }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AxisAngle {
    pub axis: Vector3<f64>,
    pub theta: f64
}

impl AxisAngle {
    pub fn new(x: f64, y: f64, z: f64, theta: f64) -> Self {
        AxisAngle { axis: Vector3::new(x, y, z), theta }
    }
}

impl From<Quaternion<f64>> for AxisAngle {
    fn from(quat: Quaternion<f64>) -> Self {
        let unit_quat = UnitQuaternion::from_quaternion(quat);
        let axis_opt = unit_quat.axis();
        let angle = unit_quat.angle();
        match axis_opt {
            Some(value) => {
                AxisAngle::new(value[0], value[1], value[2], angle)
            },
            None => {
                AxisAngle::new(0.0, 0.0, 0.0, 0.0)
            },
        }
    }
}

fn orientation_vector_theta_from_rotated_axes(
    new_x: Quaternion<f64>, new_z: Quaternion<f64>
) -> f64 {
    if 1.0 - new_z.k.abs() > ANGLE_ACCEPTANCE {
        let new_z_imag = new_z.imag();
        let new_x_imag = new_x.imag();
        let z_imag_axis = Vector3::z_axis();

        let normal_1 = new_z_imag.cross(&new_x_imag);
        let normal_2 = new_z_imag.cross(&z_imag_axis);
        let cos_theta_cand = normal_1.dot(&normal_2) / (normal_1.norm() * normal_2.norm());
        let cos_theta = match cos_theta_cand {
            val if val < -1.0 => -1.0,
            val if val > 1.0 => 1.0,
            _ => cos_theta_cand
        };

        return match cos_theta.acos() {
            val if val > ANGLE_ACCEPTANCE => {
                let new_z_imag_unit = UnitVector3::new_normalize(new_z_imag);
                let rot_quat_unit = UnitQuaternion::from_axis_angle(&new_z_imag_unit, -1.0 * val);
                let rot_quat = rot_quat_unit.quaternion();
                let z_axis_quat = Quaternion::new(0.0, 0.0, 0.0, 1.0);
                let test_z = (rot_quat * z_axis_quat) * rot_quat.conjugate();
                let test_z_imag = test_z.imag();

                let normal_3 = new_z_imag.cross(&test_z_imag);
                let cos_test = normal_1.dot(&normal_3) / (normal_3.norm() * normal_1.norm());
                match cos_test {
                    val2 if (1.0 - val2) < (ANGLE_ACCEPTANCE * ANGLE_ACCEPTANCE) => -1.0 * val,
                    _ => val
                }
            },
            _ => 0.0
        };
    }
    match new_z.k {
        val if val < 0.0 => -1.0 * new_x.j.atan2(new_x.i),
        _ => -1.0 * new_x.j.atan2(new_x.i * -1.0)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct OrientationVector {
    pub o_vector: UnitVector3<f64>,
    pub theta: f64
}

impl OrientationVector {
    pub fn new(o_x: f64, o_y: f64, o_z: f64, theta: f64) -> Self {
        let o_vector = UnitVector3::new_normalize(Vector3::new(o_x, o_y, o_z));
        OrientationVector{o_vector, theta}
    }

    pub fn to_quaternion(&self) -> Quaternion<f64> {
        let lat = self.o_vector.z.acos();
        let lon = match self.o_vector.z {
            val if 1.0 - val > ANGLE_ACCEPTANCE => self.o_vector.y.atan2(self.o_vector.x),
            _ => 0.0
        };

        // convert angles as euler angles (lon, lat, theta) to quaternion
        // using the zyz rotational order
        let s: [f64;3] = [
            (lon / 2.0).sin(),
            (lat / 2.0).sin(),
            (self.theta / 2.0).sin()
        ];

        let c: [f64;3] = [
            (lon / 2.0).cos(),
            (lat / 2.0).cos(),
            (self.theta / 2.0).cos()
        ];

        let real = c[0]*c[1]*c[2] - s[0]*c[1]*s[2];
        let i = c[0]*s[1]*s[2] - s[0]*s[1]*c[2];
        let j = c[0]*s[1]*c[2] + s[0]*s[1]*s[2];
        let k = s[0]*c[1]*c[2] + c[0]*c[1]*s[2];

        Quaternion::new(real, i, j, k)
    }
}

impl ApproxEq for OrientationVector {
    type Margin = F64Margin;

    fn approx_eq<M: Into<Self::Margin>>(self, other: Self, margin: M) -> bool {
        let margin = margin.into();
        let vec_diff = self.o_vector.into_inner() - other.o_vector.into_inner();
        vec_diff.norm_squared().approx_eq(0.0, margin) && self.theta.approx_eq(other.theta, margin)
    }
}

impl From<Quaternion<f64>> for OrientationVector {
    fn from(quat: Quaternion<f64>) -> Self {
        let x_quat = Quaternion::new(0.0, -1.0, 0.0, 0.0);
        let z_quat = Quaternion::new(0.0, 0.0, 0.0, 1.0);

        let conj = quat.conjugate();
        let new_x = (quat * x_quat) * conj;
        let new_z = (quat * z_quat) * conj;

        let o_vector = UnitVector3::new_normalize(new_z.imag());
        let theta = orientation_vector_theta_from_rotated_axes(new_x, new_z);
        Self { o_vector, theta }
    }
}

pub fn rotate_vector_by_quaternion(
    quat: &Quaternion<f64>, vector: &Vector3<f64>
) -> Vector3<f64> {
    let quat_vec = Vector3::new(quat.i, quat.j, quat.k);
    let quat_real = quat.w;
    (2.0 * quat_vec.dot(vector) * quat_vec)
    + ((quat_real * quat_real - quat_vec.norm_squared()) * vector)
    + (2.0 * quat_real) * quat_vec.cross(vector)
}

#[cfg(test)]
mod tests {
    use float_cmp::{assert_approx_eq};
    use nalgebra::{Quaternion, Vector3};

    use super::{EulerAngles, OrientationVector, rotate_vector_by_quaternion};

    fn get_quaternion_diff_norm(quat1: &Quaternion<f64>, quat2: &Quaternion<f64>) -> f64 {
        let quat_diff = quat1.coords - quat2.coords;
        quat_diff.norm_squared()
    }

    fn get_vector_diff_norm(vec1: &Vector3<f64>, vec2: &Vector3<f64>) -> f64 {
        let vec_diff = vec2 - vec1;
        vec_diff.norm_squared()
    }

    #[test]
    fn quaternion_to_orientation_vector_works() {
        let quat = Quaternion::new(
            0.7071067811865476, 0.7071067811865476, 0.0, 0.0
        );
        let expected_ov = OrientationVector::new(
            0.0, -1.0, 0.0, 1.5707963267948966
        );
        let calc_ov: OrientationVector = quat.into();
        assert_approx_eq!(OrientationVector, calc_ov, expected_ov);

        let quat2 = Quaternion::new(
            0.7071067811865476, -0.7071067811865476, 0.0, 0.0
        );
        let expected_ov2 = OrientationVector::new(
            0.0, 1.0, 0.0, -1.5707963267948966
        );
        let calc_ov2: OrientationVector = quat2.into();
        assert_approx_eq!(OrientationVector, calc_ov2, expected_ov2);

        let quat3 = Quaternion::new(
            0.96, 0.0, -0.28, 0.0
        );
        let expected_ov3 = OrientationVector::new(
            -0.5376, 0.0, 0.8432, -1.0 * std::f64::consts::PI
        );
        let calc_ov3: OrientationVector = quat3.into();
        assert_approx_eq!(OrientationVector, calc_ov3, expected_ov3);

        let quat4 = Quaternion::new(
            0.96, 0.0, 0.0, -0.28
        );
        let expected_ov4 = OrientationVector::new(
            0.0, 0.0, 1.0, -0.5675882184166557
        );
        let calc_ov4: OrientationVector = quat4.into();
        assert_approx_eq!(OrientationVector, calc_ov4, expected_ov4);

        let quat5 = Quaternion::new(
            0.96, -0.28, 0.0, 0.0
        );
        let expected_ov5 = OrientationVector::new(
            0.0, 0.5376, 0.8432, -1.5707963267948966
        );
        let calc_ov5: OrientationVector = quat5.into();
        assert_approx_eq!(OrientationVector, calc_ov5, expected_ov5);

        let quat6 = Quaternion::new(
            0.96, 0.28, 0.0, 0.0
        );
        let expected_ov6 = OrientationVector::new(
            0.0, -0.5376, 0.8432, 1.5707963267948966
        );
        let calc_ov6: OrientationVector = quat6.into();
        assert_approx_eq!(OrientationVector, calc_ov6, expected_ov6);

        let quat7 = Quaternion::new(0.5, -0.5, -0.5, -0.5);
        let expected_ov7 = OrientationVector::new(
            0.0, 1.0, 0.0, -1.0 * std::f64::consts::PI
        );
        let calc_ov7: OrientationVector = quat7.into();
        assert_approx_eq!(OrientationVector, calc_ov7, expected_ov7);

        let quat8 = Quaternion::new(
            0.816632212270443, -0.17555966025413142, 0.39198397193979817, 0.3855375485164001
        );
        let expected_ov8 = OrientationVector::new(
            0.5048437942940054, 0.5889844266763397, 0.631054742867507, 0.02
        );
        let calc_ov8: OrientationVector = quat8.into();
        assert_approx_eq!(OrientationVector, calc_ov8, expected_ov8, epsilon = 0.0001);

    }

    #[test]
    fn orientation_vector_to_quaternion_works() {
        let ov = OrientationVector::new(
            0.0, -1.0, 0.0, 1.5707963267948966
        );
        let expected_quat = Quaternion::new(
            0.7071067811865476, 0.7071067811865476, 0.0, 0.0
        );
        let calc_quat = ov.to_quaternion();
        let mut diff = get_quaternion_diff_norm(&expected_quat, &calc_quat);
        assert_approx_eq!(f64, diff, 0.0);

        let ov2 = OrientationVector::new(
            0.0, 1.0, 0.0, -1.5707963267948966
        );
        let expected_quat2 = Quaternion::new(
            0.7071067811865476, -0.7071067811865476, 0.0, 0.0
        );
        let calc_quat2 = ov2.to_quaternion();
        diff = get_quaternion_diff_norm(&expected_quat2, &calc_quat2);
        assert_approx_eq!(f64, diff, 0.0);

        let ov3 = OrientationVector::new(
            -0.5376, 0.0, 0.8432, -1.0 * std::f64::consts::PI
        );
        let expected_quat3 = Quaternion::new(
            0.96, 0.0, -0.28, 0.0
        );
        let calc_quat3 = ov3.to_quaternion();
        diff = get_quaternion_diff_norm(&expected_quat3, &calc_quat3);
        assert_approx_eq!(f64, diff, 0.0);

        let ov4 = OrientationVector::new(
            0.0, 0.0, 1.0, -0.5675882184166557
        );
        let expected_quat4 = Quaternion::new(
            0.96, 0.0, 0.0, -0.28
        );
        let calc_quat4 = ov4.to_quaternion();
        diff = get_quaternion_diff_norm(&expected_quat4, &calc_quat4);
        assert_approx_eq!(f64, diff, 0.0);

        let ov5 = OrientationVector::new(
            0.0, 0.5376, 0.8432, -1.5707963267948966
        );
        let expected_quat5 = Quaternion::new(
            0.96, -0.28, 0.0, 0.0
        );
        let calc_quat5 = ov5.to_quaternion();
        diff = get_quaternion_diff_norm(&expected_quat5, &calc_quat5);
        assert_approx_eq!(f64, diff, 0.0);

        let ov6 = OrientationVector::new(
            0.0, -0.5376, 0.8432, 1.5707963267948966
        );
        let expected_quat6 = Quaternion::new(
            0.96, 0.28, 0.0, 0.0
        );
        let calc_quat6 = ov6.to_quaternion();
        diff = get_quaternion_diff_norm(&expected_quat6, &calc_quat6);
        assert_approx_eq!(f64, diff, 0.0);

        let ov7 = OrientationVector::new(
            0.0, 1.0, 0.0, -1.0 * std::f64::consts::PI
        );
        let expected_quat7 = Quaternion::new(0.5, -0.5, -0.5, -0.5);
        let calc_quat7 = ov7.to_quaternion();
        diff = get_quaternion_diff_norm(&expected_quat7, &calc_quat7);
        assert_approx_eq!(f64, diff, 0.0);

        let ov8 = OrientationVector::new(
            0.5048437942940054, 0.5889844266763397, 0.631054742867507, 0.02
        );
        let expected_quat8 = Quaternion::new(
            0.816632212270443, -0.17555966025413142, 0.39198397193979817, 0.3855375485164001
        );
        let calc_quat8 = ov8.to_quaternion();
        diff = get_quaternion_diff_norm(&expected_quat8, &calc_quat8);
        assert_approx_eq!(f64, diff, 0.0);

    }

    #[test]
    fn euler_angles_from_quaternion_works() {
        let quat = Quaternion::new(
            0.2705980500730985, -0.6532814824381882, 0.27059805007309856, 0.6532814824381883
        );
        let euler_angles: EulerAngles = quat.into();
        assert_approx_eq!(f64, euler_angles.pitch, std::f64::consts::PI / 2.0);
        assert_approx_eq!(f64, euler_angles.yaw, std::f64::consts::PI);
        assert_approx_eq!(f64, euler_angles.roll, std::f64::consts::PI / 4.0);

        let quat2 = Quaternion::new(
            0.4619397662556435, -0.19134171618254486, 0.4619397662556434, 0.7325378163287418
        );
        let euler_angles2: EulerAngles = quat2.into();
        assert_approx_eq!(f64, euler_angles2.pitch, std::f64::consts::PI / 4.0);
        assert_approx_eq!(f64, euler_angles2.yaw, 3.0 * std::f64::consts::PI / 4.0);
        assert_approx_eq!(f64, euler_angles2.roll, std::f64::consts::PI / 4.0);
    }

    #[test]
    fn rotation_by_quaternion_works() {
        // rotation of (0,0,1) by 90 degrees about (0,1,0)
        let quat = Quaternion::new(0.7071068, 0.0, 0.7071068, 0.0);
        let vector = Vector3::new(0.0, 0.0, 1.0);
        let expected_vector = Vector3::new(1.0, 0.0, 0.0);
        let rotated_vector = rotate_vector_by_quaternion(&quat, &vector);
        let diff = get_vector_diff_norm(&expected_vector, &rotated_vector);
        assert_approx_eq!(f64, diff, 0.0, epsilon = 0.0001);

        // rotation of (4.5, 1.3, 2.0) by 175 degrees about (2,3,4)
        let quat2 = Quaternion::new(0.0436194, 0.3710372, 0.5565558, 0.7420744);
        let vector2 = Vector3::new(4.5, 1.3, 2.0);
        let expected_vector2 = Vector3::new(-1.593, 3.247, 3.586);
        let rotated_vector2 = rotate_vector_by_quaternion(&quat2, &vector2);
        let diff = get_vector_diff_norm(&expected_vector2, &rotated_vector2);
        assert_approx_eq!(f64, diff, 0.0, epsilon = 0.0001);
    }
    
}
