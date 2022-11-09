use ffi_helpers::null_pointer_check;
use libc::c_double;

use crate::spatialmath::{quaternion::Quaternion, vector3::Vector3};

/// The FFI interface for Quaternion functions and initialization. All public 
/// functions are meant to be called externally from other languages. Quaternions
/// use the Real-I-J-K standard, so quaternions in other standards should be
/// converted in the native language before being used to initialize quaternions
/// from this library

/// Allocates a copy of the quaternion to the heap with a stable memory address and
/// returns the raw pointer (for use by the FFI interface)
fn to_raw_pointer(quat: &Quaternion) -> *mut Quaternion {
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
pub extern "C" fn new_quaternion(real: f64, i: f64, j: f64, k: f64) -> *mut Quaternion {
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
    real: f64, imag_ptr: *const Vector3
) -> *mut Quaternion {
    null_pointer_check!(imag_ptr);
    to_raw_pointer(&Quaternion::new_from_vector(real, *imag_ptr))
}

/// Free memory at the address of the quaternion pointer. Outer processes
/// that work with Quaternions via the FFI interface MUST remember 
/// to call this function when finished with a quaternion
/// 
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn free_quaternion_memory(ptr: *mut Quaternion) {
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
pub unsafe extern "C" fn quaternion_get_components(quat_ptr: *const Quaternion) -> *const c_double {
    null_pointer_check!(quat_ptr);
    let components: [c_double;4] = [(*quat_ptr).real, (*quat_ptr).i, (*quat_ptr).j, (*quat_ptr).k];
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
pub unsafe extern "C" fn quaternion_set_real(quat_ptr: *mut Quaternion, real: f64) {
    null_pointer_check!(quat_ptr);
    (*quat_ptr).real = real;
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
pub unsafe extern "C" fn quaternion_set_i(quat_ptr: *mut Quaternion, i: f64) {
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
pub unsafe extern "C" fn quaternion_set_j(quat_ptr: *mut Quaternion, j: f64) {
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
pub unsafe extern "C" fn quaternion_set_k(quat_ptr: *mut Quaternion, k: f64) {
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
    quat_ptr: *mut Quaternion, real: f64, i: f64, j: f64, k: f64
) {
    null_pointer_check!(quat_ptr);
    (*quat_ptr).real = real;
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
pub unsafe extern "C" fn quaternion_set_imag_from_vector(quat_ptr: *mut Quaternion, vec_ptr: *const Vector3) {
    null_pointer_check!(quat_ptr);
    null_pointer_check!(vec_ptr);
    (*quat_ptr).set_imag_from_vector(*vec_ptr);
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
pub unsafe extern "C" fn quaternion_get_imaginary_vector(quat_ptr: *const Quaternion) -> *mut Vector3 {
    null_pointer_check!(quat_ptr);
    let imag = (*quat_ptr).imag();
    imag.to_raw_pointer()
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
pub unsafe extern "C" fn normalize_quaternion(quat_ptr: *mut Quaternion) {
    null_pointer_check!(quat_ptr);
    (*quat_ptr).normalize()
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
pub unsafe extern "C" fn quaternion_get_normalized(quat_ptr: *const Quaternion) -> *mut Quaternion {
    null_pointer_check!(quat_ptr);
    to_raw_pointer(&(*quat_ptr).get_normalized())
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
pub unsafe extern "C" fn quaternion_from_euler_angles(roll: f64, pitch: f64, yaw: f64) -> *mut Quaternion {
    let quat = Quaternion::from_euler_angles(roll, pitch, yaw);
    to_raw_pointer(&quat)
}

/// Converts a quaternion into euler angles (in radians). The euler angles are 
/// represented according to the Tait-Bryan formalism and applied 
/// in the Z-Y'-X" order (where Z -> yaw, Y -> pitch, X -> roll). 
/// The return value is a pointer to a list of [roll, pitch, yaw]
/// as C doubles
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion passed to this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn quaternion_to_euler_angles(quat_ptr: *const Quaternion) -> *const c_double {
    null_pointer_check!(quat_ptr);
    let euler_angles = (*quat_ptr).to_euler_angles();
    Box::into_raw(Box::new(euler_angles)) as *const _
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
pub unsafe extern "C" fn scale_quaternion(quat_ptr: *mut Quaternion, factor: f64) {
    null_pointer_check!(quat_ptr);
    (*quat_ptr).scale(factor);
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
pub unsafe extern "C" fn quaternion_get_scaled(quat_ptr: *const Quaternion, factor: f64) -> *mut Quaternion {
    null_pointer_check!(quat_ptr);
    to_raw_pointer(&(*quat_ptr).get_scaled(factor))
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
pub unsafe extern "C" fn quaternion_get_conjugate(quat_ptr: *const Quaternion) -> *mut Quaternion {
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
    quat_ptr_1: *const Quaternion,
    quat_ptr_2: *const Quaternion,
) -> *mut Quaternion {
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
    quat_ptr_1: *const Quaternion,
    quat_ptr_2: *const Quaternion,
) -> *mut Quaternion {
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
    quat_ptr_1: *const Quaternion,
    quat_ptr_2: *const Quaternion,
) -> *mut Quaternion {
    null_pointer_check!(quat_ptr_1);
    null_pointer_check!(quat_ptr_2);
    to_raw_pointer(&((*quat_ptr_1) * (*quat_ptr_2)))
}
