use ffi_helpers::null_pointer_check;
use libc::c_double;

use crate::spatialmath::vector3::Vector3;

/// The FFI interface for Vector functions and initialization. All public 
/// functions are meant to be called externally from other languages

/// Initialize a 3-vector from raw components and retrieve the C pointer
/// to its address.
/// 
/// # Safety
/// 
/// When finished with the underlying vector initialized by this function
/// the caller must remember to free the vector memory using the 
/// free_vector_memory FFI function
#[no_mangle]
pub extern "C" fn new_vector3(x: f64, y: f64, z: f64) -> *mut Vector3 {
    Vector3::new(x, y, z).to_raw_pointer()
}

/// Free memory at the address of the vector pointer. Outer processes
/// that work with Vectors via the FFI interface MUST remember 
/// to call this function when finished with a vector
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn free_vector_memory(ptr: *mut Vector3) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}

/// Get the components of a vector as a list of C doubles, the order of the
/// components will be (x, y, z).
/// 
/// # Safety
/// 
/// When finished with the underlying vector, the caller must remember to 
/// free the vector memory using the free_vector_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn vector_get_components(vec_ptr: *const Vector3) -> *const c_double {
    null_pointer_check!(vec_ptr);
    let components: [c_double; 3] = [(*vec_ptr).x, (*vec_ptr).y, (*vec_ptr).z];
    Box::into_raw(Box::new(components)) as *const _
}

/// Set the x component of an existing vector stored at the address
/// of a pointer.
/// 
/// # Safety
/// 
/// When finished with the underlying vector, the caller must remember to 
/// free the vector memory using the free_vector_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn vector_set_x(vec_ptr: *mut Vector3, x_val: f64) {
    null_pointer_check!(vec_ptr);
    (*vec_ptr).x = x_val;
}

/// Set the y component of an existing vector stored at the address
/// of a pointer.
/// 
/// # Safety
/// 
/// When finished with the underlying vector, the caller must remember to 
/// free the vector memory using the free_vector_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn vector_set_y(vec_ptr: *mut Vector3, y_val: f64) {
    null_pointer_check!(vec_ptr);
    (*vec_ptr).y = y_val;
}

/// Set the z component of an existing vector stored at the address
/// of a pointer.
/// 
/// # Safety
/// 
/// When finished with the underlying vector, the caller must remember to 
/// free the vector memory using the free_vector_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn vector_set_z(vec_ptr: *mut Vector3, z_val: f64) {
    null_pointer_check!(vec_ptr);
    (*vec_ptr).z = z_val;
}

/// Normalizes an existing vector stored at the address of 
/// a pointer (vec_ptr)
/// 
/// # Safety
/// 
/// When finished with the underlying vector, the caller must remember to 
/// free the vector memory using the free_vector_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn normalize_vector(vec_ptr: *mut Vector3) {
    null_pointer_check!(vec_ptr);
    (*vec_ptr).normalize();
}

/// Initializes a normalized copy of a vector stored at the
/// address of a pointer (vec_ptr) and returns a pointer to the
/// memory of the result
/// 
/// # Safety
/// 
/// The caller must remember to free the vector memory of *both* the input and
/// output vectors when finished with them using the free_vector_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn vector_get_normalized(vec_ptr: *const Vector3) -> *mut Vector3 {
    null_pointer_check!(vec_ptr);
    let vec = (*vec_ptr).get_normalized();
    vec.to_raw_pointer()
}

/// Scales an existing vector stored at the address of 
/// a pointer (vec_ptr) by a float factor
/// 
/// # Safety
/// 
/// When finished with the underlying vector, the caller must remember to 
/// free the vector memory using the free_vector_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn scale_vector(vec_ptr: *mut Vector3, factor: f64) {
    null_pointer_check!(vec_ptr);
    (*vec_ptr).scale(factor);
}

/// Initializes a scaled copy of a vector stored at the
/// address of a pointer (vec_ptr) and returns a pointer to the
/// memory of the result
/// 
/// # Safety
/// 
/// The caller must remember to free the vector memory of *both* the input and
/// output vectors when finished with them using the free_vector_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn vector_get_scaled(vec_ptr: *const Vector3, factor: f64) -> *mut Vector3 {
    null_pointer_check!(vec_ptr);
    let vec = (*vec_ptr).get_scaled(factor);
    vec.to_raw_pointer()
}

/// Adds two vectors and returns a pointer to the 
/// memory of the result
/// 
/// # Safety
/// 
/// The caller must remember to free the vector memory of *both* the input and
/// output vectors when finished with them using the free_vector_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn vector_add(
    vec_ptr_1: *const Vector3,
    vec_ptr_2: *const Vector3,
) -> *mut Vector3 {
    null_pointer_check!(vec_ptr_1);
    null_pointer_check!(vec_ptr_2);
    ((*vec_ptr_1) + (*vec_ptr_2)).to_raw_pointer()
}

/// Subtracts two vectors and returns a pointer to the 
/// memory of the result
/// 
/// # Safety
/// 
/// The caller must remember to free the vector memory of *both* the input and
/// output vectors when finished with them using the free_vector_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn vector_subtract(
    vec_ptr_1: *const Vector3,
    vec_ptr_2: *const Vector3,
) -> *mut Vector3 {
    null_pointer_check!(vec_ptr_1);
    null_pointer_check!(vec_ptr_2);
    ((*vec_ptr_1) - (*vec_ptr_2)).to_raw_pointer()
}

/// Computes the dot product of two vectors
/// 
/// # Safety
/// 
/// The caller must remember to free the vector memory of the input vectors 
/// when finished with them using the free_vector_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn vector_dot_product(
    vec_ptr_1: *const Vector3,
    vec_ptr_2: *const Vector3,
) -> f64 {
    null_pointer_check!(vec_ptr_1, f64::NAN);
    null_pointer_check!(vec_ptr_2, f64::NAN);
    (*vec_ptr_1).dot(&*vec_ptr_2)
}

/// Computes the cross product of two vectors and returns 
/// a pointer to the memory of the result
/// 
/// # Safety
/// 
/// The caller must remember to free the vector memory of *both* the input and
/// output vectors when finished with them using the free_vector_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn vector_cross_product(
    vec_ptr_1: *mut Vector3,
    vec_ptr_2: *mut Vector3,
) -> *mut Vector3 {
    null_pointer_check!(vec_ptr_1);
    null_pointer_check!(vec_ptr_2);
    let vec = (*vec_ptr_1).cross(&*vec_ptr_2);
    vec.to_raw_pointer()
}
