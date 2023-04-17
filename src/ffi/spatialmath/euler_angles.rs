use ffi_helpers::null_pointer_check;
use nalgebra::Quaternion;

use crate::spatialmath::utils::EulerAngles;

/// The FFI interface for initializing euler angles. Our euler angles
/// follow the Tait-Bryan formalism and are applied in the Z-Y'-X" order 
/// (where Z -> yaw, Y -> pitch, X -> roll).
/// 
/// It is highly recommended not to attempt any mathematics with the euler
/// angles directly and to convert to quaternions via the FFI interface instead

/// Allocates a copy of the euler angles to the heap with a stable memory address and
/// returns the raw pointer (for use by the FFI interface)
fn to_raw_pointer(ea: &EulerAngles) -> *mut EulerAngles {
    Box::into_raw(Box::new(*ea))
}

/// Free memory at the address of the euler angles pointer. 
/// 
/// # Safety
/// 
/// Outer processes that work with EulerAngles via the FFI interface MUST remember 
/// to call this function when finished with a euler angles instance
#[no_mangle]
pub unsafe extern "C" fn free_euler_angles_memory(ptr: *mut EulerAngles) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}

/// Initialize euler angles from raw components and retrieve the C pointer
/// to its address.
/// 
/// # Safety
/// 
/// When finished with the underlying euler angles initialized by this function
/// the caller must remember to free the euler angles memory using the 
/// free_euler_angles_memory FFI function
#[no_mangle]
pub extern "C" fn new_euler_angles(roll: f64, pitch: f64, yaw: f64) -> *mut EulerAngles {
    to_raw_pointer(&EulerAngles::new(roll, pitch, yaw))
}

/// Converts a quaternion into euler angles (in radians). The euler angles are 
/// represented according to the Tait-Bryan formalism and applied 
/// in the Z-Y'-X" order (where Z -> yaw, Y -> pitch, X -> roll).
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion passed to this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function and the euler angles memory using
/// the free_euler_angles_memory function
#[no_mangle]
pub unsafe extern "C" fn euler_angles_from_quaternion(quat_ptr: *const Quaternion<f64>) -> *mut EulerAngles {
    null_pointer_check!(quat_ptr);
    let euler_angles: EulerAngles = (*quat_ptr).into();
    to_raw_pointer(&euler_angles)
}
