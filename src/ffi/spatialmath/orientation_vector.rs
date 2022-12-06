use ffi_helpers::null_pointer_check;
use nalgebra::Quaternion;

use crate::spatialmath::utils::OrientationVector;

/// The FFI Interface for initialization of Viam's Orientation Vector format.
/// Like an axis angle, the format involves a vector axis and a rotation
/// theta (in radians). However, unlike an axis-angle, an orientation vector alters 
/// the axes of the given frame of reference by rotating the z-axis to the 
/// vector axis provided. The new x-axis is the vector that is both orthogonal to 
/// the vector axis provided anc co-planar with both the old
/// z-axis and the vector axis (this leaves two choices for the y-axis, 
/// but the canonical "right-hand rule" is used to select one consistently). Then, 
/// a clockwise-rotation of theta is applied about the new-z axis
/// 
/// It is highly recommended not to attempt any mathematics with the orientation
/// vector directly and to convert to quaternions via the FFI interface instead

/// Allocates a copy of the orientation vector to the heap with a stable memory address and
/// returns the raw pointer (for use by the FFI interface)
fn to_raw_pointer(o_vec: &OrientationVector) -> *mut OrientationVector {
    Box::into_raw(Box::new(*o_vec))
}

/// Free memory at the address of the orientation vector pointer. Outer processes
/// that work with OrientationVectors via the FFI interface MUST remember 
/// to call this function when finished with a OrientationVector instance
/// 
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn free_orientation_vector_memory(ptr: *mut OrientationVector) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}


/// Initialize an orientation vector from raw components and retrieve the C pointer
/// to its address.
/// 
/// # Safety
/// 
/// When finished with the underlying orientation vector initialized by this function
/// the caller must remember to free the orientation vector memory using the 
/// free_orientation_vector_memory FFI function
#[no_mangle]
pub unsafe extern "C" fn new_orientation_vector(
    o_x: f64, o_y: f64, o_z: f64, theta: f64
) -> *mut OrientationVector {
    let o_vec = OrientationVector::new(o_x, o_y, o_z, theta);
    to_raw_pointer(&o_vec)
}

/// Converts a quaternion into an orientation vector.
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion passed to this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function and the orientation-vector memory using
/// the free_orientation_vector_memory function
#[no_mangle]
pub unsafe extern "C" fn orientation_vector_from_quaternion(
    quat_ptr: *const Quaternion<f64>
) -> *mut OrientationVector {
    null_pointer_check!(quat_ptr);
    let o_vec: OrientationVector = (*quat_ptr).into();
    to_raw_pointer(&o_vec)
}
