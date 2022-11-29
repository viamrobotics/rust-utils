use nalgebra::{Quaternion, Vector3};

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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AxisAngle {
    pub vector: Vector3<f64>,
    pub theta: f64
}

impl AxisAngle {
    pub fn new(x: f64, y: f64, z: f64, theta: f64) -> Self {
        AxisAngle { vector: Vector3::new(x, y, z), theta }
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::{assert_approx_eq};
    use nalgebra::Quaternion;

    use crate::spatialmath::utils::EulerAngles;

    #[test]
    fn euler_angles_from_quaternion_works() {
        let quat = Quaternion::new(
            0.2705980500730985, -0.6532814824381882, 0.27059805007309856, 0.6532814824381883
        );
        let euler_angles = EulerAngles::from_quaternion(&quat);
        assert_approx_eq!(f64, euler_angles.pitch, std::f64::consts::PI / 2.0);
        assert_approx_eq!(f64, euler_angles.yaw, std::f64::consts::PI);
        assert_approx_eq!(f64, euler_angles.roll, std::f64::consts::PI / 4.0);

        let quat2 = Quaternion::new(
            0.4619397662556435, -0.19134171618254486, 0.4619397662556434, 0.7325378163287418
        );
        let euler_angles2 = EulerAngles::from_quaternion(&quat2);
        assert_approx_eq!(f64, euler_angles2.pitch, std::f64::consts::PI / 4.0);
        assert_approx_eq!(f64, euler_angles2.yaw, 3.0 * std::f64::consts::PI / 4.0);
        assert_approx_eq!(f64, euler_angles2.roll, std::f64::consts::PI / 4.0);
    }
}
