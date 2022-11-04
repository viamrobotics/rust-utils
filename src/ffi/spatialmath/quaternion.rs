use ffi_helpers::null_pointer_check;
use libc::c_double;

use crate::spatialmath::{quaternion::Quaternion, vector3::Vector3};

/// The FFI interface for Quaternion functions and initialization. All functions are
/// meant to be called externally from other languages

/// Allocates the quaternion to the heap with a stable memory address and
/// returns the raw pointer (for use by the FFI interface)
fn to_raw_pointer(quat: &Quaternion) -> *mut Quaternion {
    Box::into_raw(Box::new(*quat))
}

/// Initialize a quaternion from raw components and retrieve the C pointer
/// to its address 
#[no_mangle]
pub extern "C" fn new_quaternion(real: f64, i: f64, j: f64, k: f64) -> *mut Quaternion {
    to_raw_pointer(&Quaternion::new(real, i, j, k))
}

/// Initialize a quaternion from a real part and a C pointer to a Vector3
/// and retrieve the C pointer to its address 
#[no_mangle]
pub unsafe extern "C" fn new_quaternion_from_vector(
    real: f64, imag_ptr: *const Vector3
) -> *mut Quaternion {
    null_pointer_check!(imag_ptr);
    let imag = *imag_ptr;
    to_raw_pointer(&Quaternion::new_from_vector(real, imag))
}

/// Free memory at the address of the quaternion pointer. Outer processes
/// that work with Quaternions via the FFI interface MUST remember 
/// to call this function when finished with a quaternion
#[no_mangle]
pub unsafe extern "C" fn free_quaternion_memory(ptr: *mut Quaternion) {
    if ptr.is_null() {
        return;
    }
    Box::from_raw(ptr);
}

/// Get the components of a quaternion as a list of C doubles, the order of the
/// components will be (real, i, j, k)
#[no_mangle]
pub unsafe extern "C" fn quaternion_get_components(quat_ptr: *const Quaternion) -> *const c_double {
    null_pointer_check!(quat_ptr);
    let quat = *quat_ptr;
    let components: [c_double;4] = [quat.real, quat.i, quat.j, quat.k];
    Box::into_raw(Box::new(components)) as *const _
}

/// Set the real component of an existing quaternion stored at the address
/// of a pointer
#[no_mangle]
pub unsafe extern "C" fn quaternion_set_real(quat_ptr: *mut Quaternion, real: f64) {
    null_pointer_check!(quat_ptr);
    let quat = &mut*quat_ptr;
    quat.real = real
}

/// Set the i component of an existing quaternion stored at the address
/// of a pointer
#[no_mangle]
pub unsafe extern "C" fn quaternion_set_i(quat_ptr: *mut Quaternion, i: f64) {
    null_pointer_check!(quat_ptr);
    let quat = &mut*quat_ptr;
    quat.i = i
}

/// Set the j component of an existing quaternion stored at the address
/// of a pointer
#[no_mangle]
pub unsafe extern "C" fn quaternion_set_j(quat_ptr: *mut Quaternion, j: f64) {
    null_pointer_check!(quat_ptr);
    let quat = &mut*quat_ptr;
    quat.j = j
}

/// Set the k component of an existing quaternion stored at the address
/// of a pointer
#[no_mangle]
pub unsafe extern "C" fn quaternion_set_k(quat_ptr: *mut Quaternion, k: f64) {
    null_pointer_check!(quat_ptr);
    let quat = &mut*quat_ptr;
    quat.k = k;
}

/// Set all of the components of an existing quaternion stored at the address
/// of a pointer
#[no_mangle]
pub unsafe extern "C" fn quaternion_set_components(
    quat_ptr: *mut Quaternion, real: f64, i: f64, j: f64, k: f64
) {
    null_pointer_check!(quat_ptr);
    let quat = &mut*quat_ptr;
    quat.real = real;
    quat.i = i;
    quat.j = j;
    quat.k = k;
}

/// Set the imaginary components of an existing quaternion stored at
/// the address of a pointer (quat_ptr) from the components of a 3-vector
/// (stored at vec_ptr). The convention is x -> i, y -> j, z -> k
#[no_mangle]
pub unsafe extern "C" fn quaternion_set_imag_from_vector(quat_ptr: *mut Quaternion, vec_ptr: *const Vector3) {
    null_pointer_check!(quat_ptr);
    null_pointer_check!(vec_ptr);
    let mut quat = *quat_ptr;
    let imag = *vec_ptr;
    quat.set_imag_from_vector(imag);
}

/// Copies the imaginary components to a 3-vector (using x -> i, y -> j
/// z -> k) and returns a pointer to the memory address of the resulting
/// vector
#[no_mangle]
pub unsafe extern "C" fn quaternion_get_imaginary_vector(quat_ptr: *const Quaternion) -> *mut Vector3 {
    null_pointer_check!(quat_ptr);
    let quat = *quat_ptr;
    let imag = quat.imag();
    imag.to_raw_pointer()
}

/// Converts from euler angles to a quaternion. The euler angles are expected to
/// be represented according to the Tait-Bryan formalism and applied in the Z-Y'-X"
/// order (where Z -> yaw, Y -> pitch, X -> roll)
#[no_mangle]
pub unsafe extern "C" fn quaternion_from_euler_angles(roll: f64, pitch: f64, yaw: f64) -> *mut Quaternion {
    let quat = Quaternion::from_euler_angles(roll, pitch, yaw);
    to_raw_pointer(&quat)
}

/// Converts a quaternion into euler angles. The euler angles are 
/// represented according to the Tait-Bryan formalism and applied 
/// in the Z-Y'-X" order (where Z -> yaw, Y -> pitch, X -> roll). 
/// The return value is a pointer to a list of [roll, pitch, yaw]
/// as C doubles
#[no_mangle]
pub unsafe extern "C" fn quaternion_to_euler_angles(quat_ptr: *const Quaternion) -> *const c_double {
    null_pointer_check!(quat_ptr);
    let quat = *quat_ptr;
    let euler_angles = quat.to_euler_angles();
    Box::into_raw(Box::new(euler_angles)) as *const _
}
