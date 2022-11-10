use float_cmp::{ApproxEq, F64Margin, approx_eq};

use super::vector3::Vector3;

/// A Rust implementation of Quaternion, we use this instead of existing packages
/// because we want a C-safe representational structure. These quaternions use
/// the Real-I-J-K standard, so those using JPL or other standards should take
/// care to convert before initializing a Quaternion struct from the library
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Quaternion {
    pub real: f64,
    pub i: f64,
    pub j: f64,
    pub k: f64,
}

impl Quaternion {
    /// Initializes a quaternion from a real component and an
    /// imaginary 3-vector
    pub fn new_from_vector(real: f64, imag: Vector3) -> Self {
        Self{real, i: imag.x, j: imag.y, k: imag.z}
    }

    /// Initializes a Quaternion from its raw components
    pub fn new(real: f64, i: f64, j: f64, k: f64) -> Self {
        Self{real, i, j, k}
    }

    /// Converts from euler angles (in radians) to a quaternion. The euler angles 
    /// are expected to be represented according to the Tait-Bryan formalism 
    /// and applied in the Z-Y'-X" order (where Z -> yaw, Y -> pitch, X -> roll).
    pub fn from_euler_angles(roll: f64, pitch: f64, yaw: f64) -> Self {
        let roll_cos = (roll * 0.5).cos();
        let roll_sin = (roll * 0.5).sin();

        let pitch_cos = (pitch * 0.5).cos();
        let pitch_sin = (pitch * 0.5).sin();

        let yaw_cos = (yaw * 0.5).cos();
        let yaw_sin = (yaw * 0.5).sin();

        let real = (roll_cos * pitch_cos * yaw_cos) + (roll_sin * pitch_sin * yaw_sin);
        let i = (roll_sin * pitch_cos * yaw_cos) - (roll_cos * pitch_sin * yaw_sin);
        let j = (roll_cos * pitch_sin * yaw_cos) + (roll_sin * pitch_cos * yaw_sin);
        let k = (roll_cos * pitch_cos * yaw_sin) - (roll_sin * pitch_sin * yaw_cos);

        Quaternion::new(real, i, j, k)
    }

    /// Converts a quaternion into euler angles (in radians). The euler angles are 
    /// represented according to the Tait-Bryan formalism and applied 
    /// in the Z-Y'-X" order (where Z -> yaw, Y -> pitch, X -> roll). 
    /// The return value is a list of [roll, pitch, yaw] and all returned angles
    /// are in the domain of -π/2 to π/2
    pub fn to_euler_angles(&self) -> [f64;3] {
        // get a normalized version of the quaternion
        let quat = self.get_normalized();

        // calculate yaw
        let yaw_sin_pitch_cos = 2.0 * ((quat.real * quat.k) + (quat.i * quat.j));
        let yaw_cos_pitch_cos = 1.0 - 2.0 * ((quat.j * quat.j) + (quat.k * quat.k));
        let yaw = yaw_sin_pitch_cos.atan2(yaw_cos_pitch_cos);

        // calculate pitch and roll
        let pitch_sin = 2.0 * ((quat.real * quat.j) - (quat.k * quat.i));
        let pitch: f64;
        let roll: f64;
        // for a pitch that is π / 2, we experience gimbal lock
        // and must calculate roll based on the real rotation and yaw
        if pitch_sin.abs() >= 1.0 {
            pitch = (std::f64::consts::PI / 2.0).copysign(pitch_sin);
            roll = (2.0 * quat.i.atan2(quat.real)) + yaw.copysign(pitch_sin);
        } else {
            pitch = pitch_sin.asin();
            let roll_sin_pitch_cos = 2.0 * ((quat.real * quat.i) + (quat.j * quat.k));
            let roll_cos_pitch_cos = 1.0 - 2.0 * ((quat.i * quat.i) + (quat.j * quat.j));
            roll = roll_sin_pitch_cos.atan2(roll_cos_pitch_cos);
        }

        [roll, pitch, yaw]
    }

    /// Return the imaginary components of a quaternion as
    /// a 3-vector
    pub fn imag(&self) -> Vector3 {
        Vector3 { x: self.i, y: self.j, z: self.k }
    }

    /// Sets the imaginary components of a quaternion from
    /// a provided 3-vector
    pub fn set_imag_from_vector(&mut self, imag: Vector3) {
        self.i = imag.x;
        self.j = imag.y;
        self.k = imag.z;
    }

    /// Returns whether the quaternion is normalized 
    /// (a four-vector on the unit sphere in quaternion space)
    pub fn is_normalized(&self) -> bool {
        approx_eq!(f64, self.norm2().sqrt(), 1.0)
    }

    pub fn norm2(&self) -> f64 {
        (self.real * self.real) + (self.i * self.i) + (self.j * self.j) + (self.k * self.k)
    }

    /// Normalizes a quaternion
    pub fn normalize(&mut self) {
        // let inv_sq_dp = fast_inv_sqrt(self.norm2());
        self.scale(self.norm2().sqrt().recip())
    }

    /// Returns a normalized copy of the quaternion
    pub fn get_normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn scale(&mut self, factor: f64) {
        self.real *= factor;
        self.i *= factor;
        self.j *= factor;
        self.k *= factor;
    }

    pub fn get_scaled(&self, factor: f64) -> Self {
        let mut copy_quat = *self;
        copy_quat.scale(factor);
        copy_quat
    }

    pub fn conjugate(&self) -> Self {
        Self { real: self.real, i: self.i * -1.0, j: self.j * -1.0, k: self.k * -1.0 }
    }

}

impl std::ops::Add<Quaternion> for Quaternion {
    type Output = Quaternion;

    fn add(self, rhs: Quaternion) -> Quaternion {
        Self { 
            real: self.real + rhs.real, 
            i: self.i + rhs.i,
            j: self.j + rhs.j,
            k: self.k + rhs.k
        }
    }
}

impl std::ops::Sub<Quaternion> for Quaternion {
    type Output = Quaternion;

    fn sub(self, rhs: Quaternion) -> Quaternion {
        Self { 
            real: self.real - rhs.real, 
            i: self.i - rhs.i,
            j: self.j - rhs.j,
            k: self.k - rhs.k
        }
    }
}

/// Implements the Hamiltonian product for two quaternions
impl std::ops::Mul<Quaternion> for Quaternion {
    type Output = Quaternion;

    fn mul(self, _rhs: Quaternion) -> Self::Output {
        let real_0 = self.real;
        let i_0 = self.i;
        let j_0 = self.j;
        let k_0 = self.k;

        let real_1 = _rhs.real;
        let i_1 = _rhs.i;
        let j_1 = _rhs.j;
        let k_1 = _rhs.k;
        
        let real = real_0*real_1 - i_0*i_1 - j_0*j_1 - k_0*k_1;
        let i_part = real_0*i_1 + i_0*real_1 + j_0*k_1 - k_0*j_1;
        let j_part = real_0*j_1 - i_0*k_1 + j_0*real_1 + k_0*i_1;
        let k_part = real_0*k_1 + i_0*j_1 - j_0*i_1 + k_0*real_1;

        Self::new(real, i_part, j_part, k_part)
    }
}

impl ApproxEq for Quaternion {
    type Margin = F64Margin;

    fn approx_eq<M: Into<Self::Margin>>(self, other: Self, margin: M) -> bool {
        let margin = margin.into();
        let diff = other - self;
        let diff_norm2 = (
            diff.real * diff.real + diff.i * diff.i + diff.j * diff.j + diff.k * diff.k
        ).sqrt();
        diff_norm2.approx_eq(0.0, margin)
    }
}

impl PartialEq for Quaternion {
    fn eq(&self, other: &Self) -> bool {
        (self.real == other.real) && (self.i == other.i) && (self.j == other.j) && (self.k == other.k)
    }
}

#[cfg(test)]
mod tests {
    use float_cmp::{assert_approx_eq};

    use crate::spatialmath::vector3::Vector3;
    use crate::spatialmath::quaternion::Quaternion;

    #[test]
    fn new_initializes_quaternion_successfully() {
        let quat = Quaternion::new(1.0, 0.0, 0.5, 1.0);
        assert_eq!(quat.real, 1.0);
        assert_eq!(quat.i, 0.0);
        assert_eq!(quat.j, 0.5);
        assert_eq!(quat.k, 1.0)
    }

    #[test]
    fn imag_returns_imaginary_components_as_vector() {
        let quat = Quaternion::new(1.0, 0.0, 0.5, 1.0);
        let expected_imag = Vector3::new(0.0, 0.5, 1.0);
        assert_eq!(quat.imag(), expected_imag);
    }

    #[test]
    fn set_imag_from_vector_works() {
        let imag = Vector3::new(0.0, 0.5, 1.0);
        let expected_quat = Quaternion::new(1.0, 0.0, 0.5, 1.0);
        let quat = Quaternion::new_from_vector(1.0, imag);
        assert_eq!(quat, expected_quat);
    }

    #[test]
    fn quaternion_normalizes_successfully() {
        let mut quat = Quaternion::new(1.0, 2.0, 3.0, 4.0);
        quat.normalize();
        let length = quat.norm2();
        assert!(length <= 1.0);
        assert_approx_eq!(f64, length, 1.0)
    }

    #[test]
    fn get_normalized_returns_a_normalized_copy() {
        let quat = Quaternion::new(
            0.0,
            (1.0_f64 / 3.0_f64).sqrt() * 0.5, 
            (1.0_f64 / 3.0_f64).sqrt() * -0.5, 
            (1.0_f64 / 3.0_f64).sqrt() * 0.5
        );
        let expected_quat = Quaternion::new(
            0.0,
            (1.0_f64 / 3.0_f64).sqrt(), 
            (1.0_f64 / 3.0_f64).sqrt() * -1.0, 
            (1.0_f64 / 3.0_f64).sqrt()
        );
        let unchanged_quat = quat;
        let normalized_quat = quat.get_normalized();
        assert_approx_eq!(Quaternion, normalized_quat, expected_quat);
        assert_approx_eq!(Quaternion, quat, unchanged_quat);
    }

    #[test]
    fn is_normalized_is_correct() {
        let not_normalized = Quaternion::new(
            0.3,
            (1.0_f64 / 3.0_f64).sqrt() * 0.5, 
            (1.0_f64 / 3.0_f64).sqrt() * -0.5, 
            (1.0_f64 / 3.0_f64).sqrt() * 0.5
        );
        assert!(!not_normalized.is_normalized());
        let normalized = Quaternion::new(
            0.0,
            (1.0_f64 / 3.0_f64).sqrt(), 
            (1.0_f64 / 3.0_f64).sqrt() * -1.0, 
            (1.0_f64 / 3.0_f64).sqrt()
        );
        assert!(normalized.is_normalized());
    }

    #[test]
    fn add_subtract_works() {
        let quat1 = Quaternion::new(0.1, 0.2, 0.3, 0.4);
        let quat2 = Quaternion::new(0.2, 0.3, 0.4, 0.5);
        let expected_add = Quaternion::new(0.3, 0.5, 0.7, 0.9);
        let expected_sub = Quaternion::new(-0.1, -0.1, -0.1, -0.1);
        assert_approx_eq!(Quaternion, quat1 + quat2, expected_add);
        assert_approx_eq!(Quaternion, quat1 - quat2, expected_sub);
    }

    #[test]
    fn multiply_works() {
        let quat1 = Quaternion::new(0.1, 0.2, 0.3, 0.4);
        let quat2 = Quaternion::new(0.2, 0.3, 0.4, 0.5);
        let expected_mul = Quaternion::new(-0.36, 0.06, 0.12, 0.12);
        let expected_rev_mul = Quaternion::new(-0.36, 0.08, 0.08, 0.14);
        assert_approx_eq!(Quaternion, quat1 * quat2, expected_mul);
        assert_approx_eq!(Quaternion, quat2 * quat1, expected_rev_mul);
    }

    #[test]
    fn scale_works() {
        let mut quat = Quaternion::new(0.1, 0.2, 0.3, 0.4);
        quat.scale(2.0);
        let expected_quat = Quaternion::new(0.2, 0.4, 0.6, 0.8);
        assert_approx_eq!(Quaternion, quat, expected_quat);
    }

    #[test]
    fn get_scaled_returns_scaled_quaternion() {
        let quat = Quaternion::new(0.1, 0.2, 0.3, 0.4);
        let quat_orig = quat;
        let scaled_quat = quat.get_scaled(2.0);
        let expected_quat = Quaternion::new(0.2, 0.4, 0.6, 0.8);
        assert_eq!(quat, quat_orig);
        assert_approx_eq!(Quaternion, scaled_quat, expected_quat);
    }

    #[test]
    fn conjugate_works() {
        let quat = Quaternion::new(0.1, 0.2, 0.3, 0.4);
        let expected_quat = Quaternion::new(0.1, -0.2, -0.3, -0.4);
        assert_approx_eq!(Quaternion, quat.conjugate(), expected_quat);
    }

    #[test]
    fn quaternion_initializes_from_euler_angles() {
        let expected_quat = Quaternion::new(
            0.2705980500730985, -0.6532814824381882, 0.27059805007309856, 0.6532814824381883
        );
        let roll = std::f64::consts::PI / 4.0;
        let pitch = std::f64::consts::PI / 2.0;
        let yaw = std::f64::consts::PI;
        let quat = Quaternion::from_euler_angles(roll, pitch, yaw);
        assert_approx_eq!(Quaternion, quat, expected_quat);

        let expected_quat2 = Quaternion::new(
            0.4619397662556435, -0.19134171618254486, 0.4619397662556434, 0.7325378163287418
        );
        let roll2 = std::f64::consts::PI / 4.0;
        let pitch2 = std::f64::consts::PI / 4.0;
        let yaw2 = 3.0 * std::f64::consts::PI / 4.0;
        let quat2 = Quaternion::from_euler_angles(roll2, pitch2, yaw2);
        assert_approx_eq!(Quaternion, quat2, expected_quat2);
    }

    #[test]
    fn euler_angles_from_quaternion_works() {
        let quat = Quaternion::new(
            0.2705980500730985, -0.6532814824381882, 0.27059805007309856, 0.6532814824381883
        );
        let euler_angles = quat.to_euler_angles();
        let roll = euler_angles[0];
        let pitch = euler_angles[1];
        let yaw = euler_angles[2];
        assert_approx_eq!(f64, pitch, std::f64::consts::PI / 2.0);
        assert_approx_eq!(f64, yaw, std::f64::consts::PI);
        assert_approx_eq!(f64, roll, std::f64::consts::PI / 4.0);

        let quat2 = Quaternion::new(
            0.4619397662556435, -0.19134171618254486, 0.4619397662556434, 0.7325378163287418
        );
        let euler_angles2 = quat2.to_euler_angles();
        let roll2 = euler_angles2[0];
        let pitch2 = euler_angles2[1];
        let yaw2 = euler_angles2[2];
        assert_approx_eq!(f64, pitch2, std::f64::consts::PI / 4.0);
        assert_approx_eq!(f64, yaw2, 3.0 * std::f64::consts::PI / 4.0);
        assert_approx_eq!(f64, roll2, std::f64::consts::PI / 4.0);
    }
}
