use ffi_helpers::null_pointer_check;
use nalgebra::{Quaternion};

use crate::spatialmath::utils::AxisAngle;

/// The FFI interface for initializing axis angles. These are
/// R4 axis angles (meaning they are represented by the x, y, z
/// components of an axis and an additional rotational parameter
/// theta about that axis).

/// Allocates a copy of the axis angle to the heap with a stable memory address and
/// returns the raw pointer (for use by the FFI interface)
fn to_raw_pointer(aa: &AxisAngle) -> *mut AxisAngle {
    Box::into_raw(Box::new(*aa))
}

/// Free memory at the address of the axis angle pointer.
/// 
/// # Safety
/// 
/// Outer processes that work with axis angles via the FFI interface MUST remember 
/// to call this function when finished with an axis angle instance
#[no_mangle]
pub unsafe extern "C" fn free_axis_angles_memory(ptr: *mut AxisAngle) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}

/// Initialize axis angle from raw components and retrieve the C pointer
/// to its address.
/// 
/// # Safety
/// 
/// When finished with the underlying axis angle initialized by this function
/// the caller must remember to free the axis angle memory using the 
/// free_axis_angles_memory FFI function
#[no_mangle]
pub extern "C" fn new_axis_angle(x: f64, y: f64, z: f64, theta: f64) -> *mut AxisAngle {
    to_raw_pointer(&AxisAngle::new(x, y, z, theta))
}

/// Converts a quaternion into an R4 axis angle. The return value is a pointer
/// to a list of [x, y, x, theta], where (x,y,z) are the axis vector components
/// and theta is the rotation about the axis in radians. A zero quaternion returns
/// a zero axis angle. In the event of an error from the nalgebra crate, a zero
/// axis angle is also returned.
/// 
/// # Safety
/// 
/// When finished with the underlying quaternion passed to this function
/// the caller must remember to free the quaternion memory using the 
/// free_quaternion_memory FFI function and the axis angle memory using
/// the free_array_memory function
#[no_mangle]
pub unsafe extern "C" fn axis_angle_from_quaternion(
    quat: *const Quaternion<f64>
) -> *mut AxisAngle {
    null_pointer_check!(quat);
    let axis_angle = match (*quat).try_into() {
        Ok(aa) => aa,
        Err(_err) => AxisAngle::new(0.0, 0.0, 0.0, 0.0)
    };
    to_raw_pointer(&axis_angle)
}
