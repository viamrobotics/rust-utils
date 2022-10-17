use ffi_helpers::null_pointer_check;
use libc::c_double;

use crate::spatialmath::vector3::Vector3;

#[no_mangle]
pub extern "C" fn new_vector3(x: f64, y: f64, z: f64) -> *mut Vector3 {
    Vector3::new(x, y, z).to_raw_pointer()
}

#[no_mangle]
pub extern "C" fn free_vector_memory(ptr: *mut Vector3) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        Box::from_raw(ptr);
    }
}

#[no_mangle]
pub unsafe extern "C" fn vector_get_components(vec_ptr: *const Vector3) -> *const c_double {
    null_pointer_check!(vec_ptr);
    let vec = *vec_ptr;
    let components: [c_double;3] = [vec.x, vec.y, vec.z];
    Box::into_raw(Box::new(components)) as *const _
}

#[no_mangle]
pub unsafe extern "C" fn vector_set_x(vec_ptr: *mut Vector3, x_val: f64) {
    null_pointer_check!(vec_ptr);
    let vec = &mut*vec_ptr;
    vec.x = x_val
}

#[no_mangle]
pub unsafe extern "C" fn vector_set_y(vec_ptr: *mut Vector3, y_val: f64) {
    null_pointer_check!(vec_ptr);
    let vec = &mut*vec_ptr;
    vec.y = y_val
}

#[no_mangle]
pub unsafe extern "C" fn vector_set_z(vec_ptr: *mut Vector3, z_val: f64) {
    null_pointer_check!(vec_ptr);
    let vec = &mut*vec_ptr;
    vec.z = z_val
}

#[no_mangle]
pub unsafe extern "C" fn normalize_vector(vec_ptr: *mut Vector3) {
    null_pointer_check!(vec_ptr);
    let vec = &mut*vec_ptr;
    vec.normalize();
}

#[no_mangle]
pub unsafe extern "C" fn vector_get_normalized(vec_ptr: *const Vector3) -> *mut Vector3 {
    null_pointer_check!(vec_ptr);
    let vec1 = &*vec_ptr;
    let vec = vec1.get_normalized();
    return vec.to_raw_pointer()
}

#[no_mangle]
pub unsafe extern "C" fn scale_vector(vec_ptr: *mut Vector3, factor: f64) {
    null_pointer_check!(vec_ptr);
    let vec = &mut*vec_ptr;
    vec.scale(factor);
}

#[no_mangle]
pub unsafe extern "C" fn vector_get_scaled(vec_ptr: *const Vector3, factor: f64) -> *mut Vector3 {
    null_pointer_check!(vec_ptr);
    let vec1 = &*vec_ptr;
    let vec = vec1.get_scaled(factor);
    return vec.to_raw_pointer()
}

#[no_mangle]
pub unsafe extern "C" fn vector_add(
    vec_ptr_1: *const Vector3, vec_ptr_2: *const Vector3
) -> *mut Vector3 {
    null_pointer_check!(vec_ptr_1);
    null_pointer_check!(vec_ptr_2);
    let vec1 = &*vec_ptr_1;
    let vec2 = &*vec_ptr_2;
    return (*vec1 + *vec2).to_raw_pointer()
}

#[no_mangle]
pub unsafe extern "C" fn vector_subtract(
    vec_ptr_1: *const Vector3, vec_ptr_2: *const Vector3
) -> *mut Vector3 {
    null_pointer_check!(vec_ptr_1);
    null_pointer_check!(vec_ptr_2);
    let vec1 = &*vec_ptr_1;
    let vec2 = &*vec_ptr_2;
    return (*vec1 - *vec2).to_raw_pointer()
}

#[no_mangle]
pub unsafe extern "C" fn vector_dot_product(vec_ptr_1: *const Vector3, vec_ptr_2: *const Vector3) -> f64 {
    null_pointer_check!(vec_ptr_1, f64::NAN);
    null_pointer_check!(vec_ptr_2, f64::NAN);
    let vec1 = &*vec_ptr_1;
    let vec2 = &*vec_ptr_2;
    return vec1.dot(vec2);
}

#[no_mangle]
pub unsafe extern "C" fn vector_cross_product(
    vec_ptr_1: *mut Vector3, vec_ptr_2: *mut Vector3
) -> *mut Vector3 {
    null_pointer_check!(vec_ptr_1);
    null_pointer_check!(vec_ptr_2);
    let vec1 = &mut*vec_ptr_1;
    let vec2 = &mut*vec_ptr_2;
    let vec = vec1.cross(vec2);
    return vec.to_raw_pointer()
}
