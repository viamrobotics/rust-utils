use ffi_helpers::null_pointer_check;
use nalgebra::{Quaternion, UnitQuaternion};

use crate::spatialmath::utils::AxisAngle;

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
/// a zero axis angle
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
    let unit_quat = UnitQuaternion::from_quaternion(*quat);
    let axis_opt = unit_quat.axis();
    let angle = unit_quat.angle();
    let axis_angle = match axis_opt {
        Some(value) => {
            AxisAngle::new(value[0], value[1], value[2], angle)
        },
        None => {
            AxisAngle::new(0.0, 0.0, 0.0, 0.0)
        },
    };
    to_raw_pointer(&axis_angle)
}
