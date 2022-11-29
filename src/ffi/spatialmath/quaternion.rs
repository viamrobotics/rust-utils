use ffi_helpers::null_pointer_check;
use libc::c_double;
use nalgebra::{Quaternion, Vector3, UnitQuaternion, Normed, UnitVector3};

use crate::ffi::spatialmath::vector3::to_raw_pointer as vec_to_raw_pointer;

/// The FFI interface wrapper around the nalgebra crate for Quaternion functions 
/// and initialization. All public functions are meant to be called externally 
/// from other languages. Quaternions
/// use the Real-I-J-K standard, so quaternions in other standards should be
/// converted in the native language before being used to initialize quaternions
/// from this library

/// Allocates a copy of the quaternion to the heap with a stable memory address and
/// returns the raw pointer (for use by the FFI interface)
fn to_raw_pointer(quat: &Quaternion<f64>) -> *mut Quaternion<f64> {
    Box::into_raw(Box::new(*quat))
}

/// Initialize a quaternion from raw components and retrieve the C pointer
/// to its address.
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion initialized by this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function
#[no_mangle]
pub extern "C" fn new_quaternion(real: f64, i: f64, j: f64, k: f64) -> *mut Quaternion<f64> {
    to_raw_pointer(&Quaternion::new(real, i, j, k))
}

/// Initialize a quaternion from a real part and a C pointer to a Vector3
/// and retrieve the C pointer to its address.
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion initialized by this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn new_quaternion_from_vector(
    real: f64, imag_ptr: *const Vector3<f64>
) -> *mut Quaternion<f64> {
    null_pointer_check!(imag_ptr);
    to_raw_pointer(&Quaternion::new(real, (*imag_ptr).x, (*imag_ptr).y, (*imag_ptr).z))
}

/// Free memory at the address of the quaternion pointer. 
/// 
/// # Safety
/// 
/// Outer processes that work with Quaternions via the FFI interface MUST remember 
/// to call this function when finished with a quaternion
#[no_mangle]
pub unsafe extern "C" fn free_quaternion_memory(ptr: *mut Quaternion<f64>) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}

/// Get the components of a quaternion as a list of C doubles, the order of the
/// components will be (real, i, j, k).
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion passed to this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_get_components(quat_ptr: *const Quaternion<f64>) -> *const c_double {
    null_pointer_check!(quat_ptr);
    let components: [c_double;4] = [(*quat_ptr).w, (*quat_ptr).i, (*quat_ptr).j, (*quat_ptr).k];
    Box::into_raw(Box::new(components)) as *const _
}

/// Set the real component of an existing quaternion stored at the address
/// of a pointer.
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion passed to this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_set_real(quat_ptr: *mut Quaternion<f64>, real: f64) {
    null_pointer_check!(quat_ptr);
    (*quat_ptr).w = real;
}

/// Set the i component of an existing quaternion stored at the address
/// of a pointer.
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion passed to this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_set_i(quat_ptr: *mut Quaternion<f64>, i: f64) {
    null_pointer_check!(quat_ptr);
    (*quat_ptr).i = i;
}

/// Set the j component of an existing quaternion stored at the address
/// of a pointer.
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion passed to this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_set_j(quat_ptr: *mut Quaternion<f64>, j: f64) {
    null_pointer_check!(quat_ptr);
    (*quat_ptr).j = j;
}

/// Set the k component of an existing quaternion stored at the address
/// of a pointer.
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion passed to this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_set_k(quat_ptr: *mut Quaternion<f64>, k: f64) {
    null_pointer_check!(quat_ptr);
    (*quat_ptr).k = k;
}

/// Set all of the components of an existing quaternion stored at the address
/// of a pointer
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion passed to this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_set_components(
    quat_ptr: *mut Quaternion<f64>, real: f64, i: f64, j: f64, k: f64
) {
    null_pointer_check!(quat_ptr);
    (*quat_ptr).w = real;
    (*quat_ptr).i = i;
    (*quat_ptr).j = j;
    (*quat_ptr).k = k;
}

/// Set the imaginary components of an existing quaternion stored at
/// the address of a pointer (quat_ptr) from the components of a 3-vector
/// (stored at vec_ptr). The convention is x -> i, y -> j, z -> k
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion passed to this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function (the same applies for the vector
/// stored at vec_ptr)
#[no_mangle]
pub unsafe extern "C" fn quaternion_set_imag_from_vector(quat_ptr: *mut Quaternion<f64>, vec_ptr: *const Vector3<f64>) {
    null_pointer_check!(quat_ptr);
    null_pointer_check!(vec_ptr);
    (*quat_ptr).i = (*vec_ptr).x;
    (*quat_ptr).j = (*vec_ptr).y;
    (*quat_ptr).k = (*vec_ptr).z;
}

/// Copies the imaginary components to a 3-vector (using x -> i, y -> j
/// z -> k) and returns a pointer to the memory address of the resulting
/// vector
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion initialized by this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_get_imaginary_vector(quat_ptr: *const Quaternion<f64>) -> *mut Vector3<f64> {
    null_pointer_check!(quat_ptr);
    let imag = (*quat_ptr).vector();
    let imag_vec = Vector3::new(imag[0], imag[1], imag[2]);
    vec_to_raw_pointer(imag_vec)
}

/// Normalizes an existing quaternion stored at the address of 
/// a pointer (quat_ptr)
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion passed to this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn normalize_quaternion(quat_ptr: *mut Quaternion<f64>) {
    null_pointer_check!(quat_ptr);
    (*quat_ptr).normalize_mut();
}

/// Initializes a normalized copy of a quaternion stored at the
/// address of a pointer (quat_ptr) and returns a pointer to the
/// memory of the result
/// 
/// # Safety
/// 
/// The caller must remember to free the quaternion memory of 
/// *both* the input and output quaternions when finished with them 
/// using the free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_get_normalized(quat_ptr: *const Quaternion<f64>) -> *mut Quaternion<f64> {
    null_pointer_check!(quat_ptr);
    to_raw_pointer(&(*quat_ptr).normalize())
}

/// Converts from euler angles (in radians) to a quaternion. The euler angles are expected to
/// be represented according to the Tait-Bryan formalism and applied in the Z-Y'-X"
/// order (where Z -> yaw, Y -> pitch, X -> roll)
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion initialized by this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_from_euler_angles(roll: f64, pitch: f64, yaw: f64) -> *mut Quaternion<f64> {
    let unit_quat = UnitQuaternion::from_euler_angles(roll, pitch, yaw);
    let quat = unit_quat.quaternion();
    to_raw_pointer(&quat)
}

/// Converts from an axis angle given by a vector's x, y, z components
/// and a rotation theta (in radians) about the vector into a quaternion
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion initialized by this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_from_axis_angle(
    x: f64, y: f64, z: f64, theta: f64
) -> *mut Quaternion<f64> {
    let axis_angle_vec = Vector3::new(x, y, z);
    let axis_angle_vec_normed = UnitVector3::new_normalize(axis_angle_vec);
    let unit_quat = UnitQuaternion::from_axis_angle(&axis_angle_vec_normed, theta);
    to_raw_pointer(unit_quat.quaternion())
}

/// Converts from an axis angle whose vector is given by a pointer 
/// to a nalgebra::Vector3<f64> instance and a rotation theta (in radians)
/// about the vector to a quaternion
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion initialized by this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function. Similarly the free_vector_memory should
/// be called when finished with the axis angle vector
#[no_mangle]
pub unsafe extern "C" fn quaternion_from_axis_angle_vector(
    theta: f64, axis_angle_vec_ptr: *const Vector3<f64>
) -> *mut Quaternion<f64> {
    null_pointer_check!(axis_angle_vec_ptr);
    let axis_angle_vec_normed = UnitVector3::new_normalize(*axis_angle_vec_ptr);
    let unit_quat = UnitQuaternion::from_axis_angle(&axis_angle_vec_normed, theta);
    to_raw_pointer(unit_quat.quaternion())
}

/// Scales an existing quaternion stored at the address of 
/// a pointer (quat_ptr) by a factor (float)
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion passed to this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn scale_quaternion(quat_ptr: *mut Quaternion<f64>, factor: f64) {
    null_pointer_check!(quat_ptr);
    (*quat_ptr).scale_mut(factor);
}

/// Initializes a copy of the quaternion stored at the address of a pointer (quat_ptr)
/// scaled by a factor (float) and returns a pointer to the memory of the result
/// 
/// # Safety
/// 
/// The caller must remember to free the quaternion memory of 
/// *both* the input and output quaternions when finished with them 
/// using the free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_get_scaled(quat_ptr: *const Quaternion<f64>, factor: f64) -> *mut Quaternion<f64> {
    null_pointer_check!(quat_ptr);
    let mut copy_quat = *quat_ptr;
    copy_quat.scale_mut(factor);
    to_raw_pointer(&copy_quat)
}

/// Initializes a quaternion that is the conjugate of one stored 
/// at the address of a pointer (quat_ptr)and returns a pointer 
/// to the memory of the result
/// 
/// # Safety
/// 
/// The caller must remember to free the quaternion memory of 
/// *both* the input and output quaternions when finished with them 
/// using the free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_get_conjugate(quat_ptr: *const Quaternion<f64>) -> *mut Quaternion<f64> {
    null_pointer_check!(quat_ptr);
    to_raw_pointer(&(*quat_ptr).conjugate())
}

/// Adds two quaternions and returns a pointer to the 
/// memory of the result
/// 
/// # Safety
/// 
/// The caller must remember to free the quaternion memory of *both* the input and
/// output quaternions when finished with them using the free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_add(
    quat_ptr_1: *const Quaternion<f64>,
    quat_ptr_2: *const Quaternion<f64>,
) -> *mut Quaternion<f64> {
    null_pointer_check!(quat_ptr_1);
    null_pointer_check!(quat_ptr_2);
    to_raw_pointer(&((*quat_ptr_1) + (*quat_ptr_2)))
}

/// Subtracts two quaternions and returns a pointer to the 
/// memory of the result
/// 
/// # Safety
/// 
/// The caller must remember to free the quaternion memory of *both* the input and
/// output quaternions when finished with them using the free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_subtract(
    quat_ptr_1: *const Quaternion<f64>,
    quat_ptr_2: *const Quaternion<f64>,
) -> *mut Quaternion<f64> {
    null_pointer_check!(quat_ptr_1);
    null_pointer_check!(quat_ptr_2);
    to_raw_pointer(&((*quat_ptr_1) - (*quat_ptr_2)))
}

/// Computes the Hamiltonian product of two quaternions and 
/// returns a pointer to the memory of the result
/// 
/// # Safety
/// 
/// The caller must remember to free the quaternion memory of *both* the input and
/// output quaternions when finished with them using the free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_hamiltonian_product(
    quat_ptr_1: *const Quaternion<f64>,
    quat_ptr_2: *const Quaternion<f64>,
) -> *mut Quaternion<f64> {
    null_pointer_check!(quat_ptr_1);
    null_pointer_check!(quat_ptr_2);
    to_raw_pointer(&((*quat_ptr_1) * (*quat_ptr_2)))
}
