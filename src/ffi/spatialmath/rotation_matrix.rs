use ffi_helpers::null_pointer_check;
use libc::c_double;
use nalgebra::{Matrix3, Quaternion, Rotation3, UnitQuaternion};

/// The FFI interface wrapper around the nalgebra crate for RotationMatrix functions
/// and initialization. All public functions are meant to be called externally
/// from other languages. These are 3D rotations (so members of SO(3))

/// Allocates a copy of the rotation matrix to the heap with a stable memory address and
/// returns the raw pointer (for use by the FFI interface)
fn to_raw_pointer(rot: &Rotation3<f64>) -> *mut Rotation3<f64> {
    Box::into_raw(Box::new(*rot))
}

/// Free memory at the address of the rotation matrix pointer. Outer processes
/// that work with RotationMatrices via the FFI interface MUST remember
/// to call this function when finished with a rotation matrix
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn viam_free_rotation_matrix_memory(ptr: *mut Rotation3<f64>) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
#[deprecated]
pub unsafe extern "C" fn free_rotation_matrix_memory(ptr: *mut Rotation3<f64>) {
    viam_free_rotation_matrix_memory(ptr)
}

/// Initialize a 3D rotation matrix from raw components and retrieve the C pointer
/// to its address. Elements are interpreted in row-major order: elements[3*r + c]
/// is the element in row r and column c. This function DOES NOT check whether
/// the matrix elements provided form a valid member of SO(3)
///
/// # Safety
///
/// When finished with the underlying rotation matrix initialized by this function
/// the caller must remember to free the rotation matrix memory using the
/// free_rotation_matrix_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn viam_new_rotation_matrix(
    elements: *const [f64; 9],
) -> *mut Rotation3<f64> {
    null_pointer_check!(elements);
    let e = *elements;
    let matrix = Matrix3::new(
        e[0], e[1], e[2],
        e[3], e[4], e[5],
        e[6], e[7], e[8],
    );
    let rot = Rotation3::from_matrix_unchecked(matrix);
    to_raw_pointer(&rot)
}

#[no_mangle]
#[deprecated]
pub unsafe extern "C" fn new_rotation_matrix(elements: *const [f64; 9]) -> *mut Rotation3<f64> {
    viam_new_rotation_matrix(elements)
}

/// Converts a quaternion into a 3D rotation matrix (a Rotation<f64, 3>
/// from the nalgebra crate)
///
/// # Safety
///
/// When finished with the underlying quaternion passed to this function
/// the caller must remember to free the quaternion memory using the
/// free_quaternion_memory FFI function and the rotation matrix memory using
/// the free_rotation_matrix_memory function
#[no_mangle]
pub unsafe extern "C" fn viam_rotation_matrix_from_quaternion(
    quat: *const Quaternion<f64>,
) -> *mut Rotation3<f64> {
    null_pointer_check!(quat);
    let unit_quat = UnitQuaternion::new_normalize(*quat);
    let rot = unit_quat.to_rotation_matrix();
    to_raw_pointer(&rot)
}

#[no_mangle]
#[deprecated]
pub unsafe extern "C" fn rotation_matrix_from_quaternion(
    quat: *const Quaternion<f64>,
) -> *mut Rotation3<f64> {
    viam_rotation_matrix_from_quaternion(quat)
}

/// Returns the elements in row-major order: index 3*r + c is R[r][c]. Caller
/// must free with viam_free_rotation_matrix_elements.
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn viam_rotation_matrix_get_elements(
    rot_ptr: *const Rotation3<f64>,
) -> *const c_double {
    null_pointer_check!(rot_ptr);
    let m = (*rot_ptr).matrix();
    let elements: [c_double; 9] = [
        m[(0, 0)], m[(0, 1)], m[(0, 2)],
        m[(1, 0)], m[(1, 1)], m[(1, 2)],
        m[(2, 0)], m[(2, 1)], m[(2, 2)],
    ];
    Box::into_raw(Box::new(elements)) as *const _
}

#[no_mangle]
#[deprecated]
pub unsafe extern "C" fn rotation_matrix_get_elements(
    rot_ptr: *const Rotation3<f64>,
) -> *const c_double {
    viam_rotation_matrix_get_elements(rot_ptr)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn viam_free_rotation_matrix_elements(ptr: *mut c_double) {
    if ptr.is_null() {
        return;
    }
    let ptr = ptr as *mut [c_double; 9];
    let _: Box<[c_double; 9]> = Box::from_raw(ptr);
}

#[no_mangle]
#[deprecated]
pub unsafe extern "C" fn free_rotation_matrix_elements(ptr: *mut c_double) {
    viam_free_rotation_matrix_elements(ptr)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Read 9 elements out of the row-major getter and free them.
    unsafe fn read_elements(rot: *const Rotation3<f64>) -> [f64; 9] {
        let ptr = viam_rotation_matrix_get_elements(rot) as *mut c_double;
        let out = *(ptr as *const [f64; 9]);
        viam_free_rotation_matrix_elements(ptr);
        out
    }

    #[test]
    fn new_and_get_elements_round_trip_row_major() {
        // 90 deg about Z, row-major.
        let elements: [f64; 9] = [0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        unsafe {
            let rot = viam_new_rotation_matrix(&elements);
            let got = read_elements(rot);
            for i in 0..9 {
                assert!((got[i] - elements[i]).abs() < 1e-12, "index {i}: got {}, want {}", got[i], elements[i]);
            }
            viam_free_rotation_matrix_memory(rot);
        }
    }

    #[test]
    fn quaternion_to_rotation_matrix_returns_row_major() {
        // Standard quat for 90 deg about Z: (w=cos(pi/4), x=0, y=0, z=sin(pi/4)).
        let half = std::f64::consts::FRAC_PI_4;
        let q = Quaternion::new(half.cos(), 0.0, 0.0, half.sin());
        unsafe {
            let rot = viam_rotation_matrix_from_quaternion(&q);
            let got = read_elements(rot);
            // Standard row-major 90 deg Z rotation:
            //   0 -1 0
            //   1  0 0
            //   0  0 1
            let want: [f64; 9] = [0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
            for i in 0..9 {
                assert!((got[i] - want[i]).abs() < 1e-12, "index {i}: got {}, want {}", got[i], want[i]);
            }
            viam_free_rotation_matrix_memory(rot);
        }
    }

    #[test]
    fn asymmetric_matrix_survives_round_trip() {
        // All off-diagonal entries populated (30 deg X composed with 45 deg Z) so any
        // sign or transposition error would produce a mismatch.
        let elements: [f64; 9] = [
            0.70710678, -0.61237244,  0.35355339,
            0.70710678,  0.61237244, -0.35355339,
            0.0,         0.5,         0.8660254,
        ];
        unsafe {
            let rot = viam_new_rotation_matrix(&elements);
            let got = read_elements(rot);
            for i in 0..9 {
                assert!((got[i] - elements[i]).abs() < 1e-8, "index {i}: got {}, want {}", got[i], elements[i]);
            }
            viam_free_rotation_matrix_memory(rot);
        }
    }
}
