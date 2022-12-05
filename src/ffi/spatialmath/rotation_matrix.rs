use ffi_helpers::null_pointer_check;
use nalgebra::{Rotation3, Quaternion, UnitQuaternion, Matrix3};

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
pub unsafe extern "C" fn free_rotation_matrix_memory(ptr: *mut Rotation3<f64>) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}

/// Initialize a 3D rotation matrix from raw components and retrieve the C pointer
/// to its address. This function DOES NOT check whether the matrix elements provided
/// form a valid member of SO(3)
/// 
/// # Safety
/// 
/// When finished with the underlying rotation matrix initialized by this function
/// the caller must remember to free the rotation matrix memory using the 
/// free_rotation_matrix_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn new_rotation_matrix(elements: *const [f64;9]) -> *mut Rotation3<f64> {
    null_pointer_check!(elements);
    let matrix = Matrix3::from_vec(Vec::from(*elements));
    let rot = Rotation3::from_matrix_unchecked(matrix);
    to_raw_pointer(&rot)
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
pub unsafe extern "C" fn rotation_matrix_from_quaternion(quat: *const Quaternion<f64>) -> *mut Rotation3<f64> {
    null_pointer_check!(quat);
    let unit_quat = UnitQuaternion::new_normalize(*quat);
    let rot = unit_quat.to_rotation_matrix();
    to_raw_pointer(&rot)
}