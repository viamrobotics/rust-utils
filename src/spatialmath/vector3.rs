use float_cmp::{ApproxEq, F64Margin, approx_eq};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vector3{x, y, z}
    }

    fn norm2(&self) -> f64 {
        self.dot(self)
    }

    pub fn get_normalized(&self) -> Self {
        let mut copy_vec = self.clone();
        copy_vec.normalize();
        copy_vec
    }

    pub fn normalize(&mut self) {
        if !self.is_normalized() {
            let norm = self.norm2().sqrt();
            if !approx_eq!(f64, norm, 0.0) {
                self.x = self.x / norm;
                self.y = self.y / norm;
                self.z = self.z / norm;
            }
        }
    }

    pub fn is_normalized(&self) -> bool {
        approx_eq!(f64, self.norm2(), 1.0)
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x, 
        )
    }

    pub fn scale(&mut self, factor: f64) {
        self.x = self.x * factor;
        self.y = self.y * factor;
        self.z = self.z * factor;
    }

    pub fn get_scaled(&self, factor: f64) -> Self {
        let mut copy_vec = self.clone();
        copy_vec.scale(factor);
        copy_vec
    }

    /// Allocates the vector to the heap with a stable memory address and
    /// returns the raw pointer (for use by the FFI interface)
    pub(crate) fn to_raw_pointer(&self) -> *mut Self {
        Box::into_raw(Box::new(*self))
    }

}

impl std::ops::Add<Vector3> for Vector3 {
    type Output = Vector3;
    fn add(self, _rhs: Vector3) -> Vector3 {
        Self { x: self.x + _rhs.x, y: self.y + _rhs.y, z: self.z + _rhs.z }
    }
}

impl std::ops::Sub<Vector3> for Vector3 {
    type Output = Vector3;
    fn sub(self, _rhs: Vector3) -> Vector3 {
        Self { x: self.x - _rhs.x, y: self.y - _rhs.y, z: self.z - _rhs.z }
    }
}

impl ApproxEq for Vector3 {
    type Margin = F64Margin;

    fn approx_eq<T: Into<Self::Margin>>(self, other: Self, margin: T) -> bool {
        let margin = margin.into();
        let diff = other - self;
        diff.norm2().sqrt().approx_eq(0.0, margin)
    }
}

impl PartialEq for Vector3 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

#[cfg(test)]
mod tests {
    use crate::spatialmath::vector3::Vector3;
    use float_cmp::approx_eq;

    #[test]
    fn new_initializes_vector_successfully() {
        let vector = Vector3::new(1.0, 1.0, 1.0);
        assert_eq!(vector.x, 1.0);
        assert_eq!(vector.y, 1.0);
        assert_eq!(vector.z, 1.0);
    }

    #[test]
    fn is_normalized_is_correct() {
        let normalized_vector = Vector3::new(
            (1.0_f64 / 3.0_f64).sqrt(), 
            (1.0_f64 / 3.0_f64).sqrt() * -1.0, 
            (1.0_f64 / 3.0_f64).sqrt());
        assert!(normalized_vector.is_normalized());

        let not_normalized_vector = Vector3::new(
            (1.0_f64 / 3.0_f64).sqrt() * 3.0, 
            (1.0_f64 / 3.0_f64).sqrt() * -3.0, 
            (1.0_f64 / 3.0_f64).sqrt() * 3.0);
        assert!(!not_normalized_vector.is_normalized());
    }

    #[test]
    fn vector_normalizes_successfully() {
        let mut vector = Vector3::new(
            (1.0_f64 / 3.0_f64).sqrt() * 3.0, 
            (1.0_f64 / 3.0_f64).sqrt() * -3.0, 
            (1.0_f64 / 3.0_f64).sqrt() * 3.0);
        let expected_vector = Vector3::new(
            (1.0_f64 / 3.0_f64).sqrt(), 
            (1.0_f64 / 3.0_f64).sqrt() * -1.0, 
            (1.0_f64 / 3.0_f64).sqrt());
        vector.normalize();
        assert!(approx_eq!(Vector3, expected_vector, vector));
    }

    #[test]
    fn get_normalized_returns_a_new_normalized_vector() {
        let vector = Vector3::new(
            (1.0_f64 / 3.0_f64).sqrt() * 3.0, 
            (1.0_f64 / 3.0_f64).sqrt() * -3.0, 
            (1.0_f64 / 3.0_f64).sqrt() * 3.0);
        let expected_vector = Vector3::new(
            (1.0_f64 / 3.0_f64).sqrt(), 
            (1.0_f64 / 3.0_f64).sqrt() * -1.0, 
            (1.0_f64 / 3.0_f64).sqrt());
        let result_vector = vector.get_normalized();
        assert!(approx_eq!(Vector3, expected_vector, result_vector));
        assert!(!vector.is_normalized());
        assert!(result_vector.is_normalized())
    }

    #[test]
    fn add_subtract_works() {
        let vector = Vector3::new(1.0, 2.0, 3.0);
        let vector2 = Vector3::new(2.0, 3.0, 1.0);
        let expected_add = Vector3::new(3.0, 5.0, 4.0);
        let expected_sub = Vector3::new(-1.0, -1.0, 2.0);
        assert_eq!(vector + vector2, expected_add);
        assert_eq!(vector - vector2, expected_sub);
    }

    #[test]
    fn scale_works() {
        let mut vector = Vector3::new(1.0, 2.0, 3.0);
        let factor = 2.0;
        let expected = Vector3::new(2.0, 4.0, 6.0);
        vector.scale(factor);
        assert_eq!(vector, expected)
    }

    #[test]
    fn get_scaled_returns_scaled_vector() {
        let vector = Vector3::new(1.0, 2.0, 3.0);
        let old_vector =  Vector3::new(1.0, 2.0, 3.0);
        let factor = 2.0;
        let expected = Vector3::new(2.0, 4.0, 6.0);
        assert_eq!(vector.get_scaled(factor), expected);
        assert_eq!(vector, old_vector);
    }

    #[test]
    fn dot_returns_dot_product() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(3.0, 4.0, 5.0);
        let expected = 26.0;
        let v1v2 = v1.dot(&v2);
        assert_eq!(v1v2, expected);
    }

    #[test]
    fn cross_product_of_parallel_vectors_returns_zero() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(2.0, 4.0, 6.0);
        let zero_v = Vector3::new(0.0,0.0,0.0);
        let v1xv2 = v1.cross(&v2);
        assert!(approx_eq!(Vector3, zero_v, v1xv2));
    }

    #[test]
    fn cross_returns_cross_product() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(3.0, 4.0, 5.0);
        let expected_vector = Vector3::new(-2.0, 4.0, -2.0);
        let v1xv2 = v1.cross(&v2);
        assert!(approx_eq!(Vector3, expected_vector, v1xv2));
    }
}