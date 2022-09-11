use crate::out;
use crate::traits::GodotClass;
use gdext_sys as sys;

use std::any::type_name;
use std::cell;

/// Manages storage and lifecycle of user's extension class instances.
pub struct InstanceStorage<T: GodotClass> {
    // FIXME should be RefCell, to avoid multi-aliasing (mut borrows from multiple shared Obj<T>)
    user_instance: cell::RefCell<T>,

    // Declared after `user_instance`, is dropped last
    pub lifecycle: Lifecycle,
    godot_ref_count: i32,

    _last_drop: LastDrop,
}

#[derive(Copy, Clone, Debug)]
pub enum Lifecycle {
    Alive,
    Destroying,
    Dead, // reading this would typically already be too late, only best-effort in case of UB
}

struct LastDrop;
impl Drop for LastDrop {
    fn drop(&mut self) {
        println!("LAST DROP");
    }
}

/// For all Godot extension classes
impl<T: GodotClass> InstanceStorage<T> {
    pub fn construct(user_instance: T) -> Self {
        out!("    Storage::construct             <{}>", type_name::<T>());

        Self {
            user_instance: cell::RefCell::new(user_instance),
            lifecycle: Lifecycle::Alive,
            godot_ref_count: 1,
            _last_drop: LastDrop,
        }
    }

    pub(crate) fn on_inc_ref(&mut self) {
        self.godot_ref_count += 1;
        out!(
            "    Storage::on_inc_ref (rc={})     <{}> -- {:?}",
            self.godot_ref_count,
            type_name::<T>(),
            self.user_instance
        );
    }

    pub(crate) fn on_dec_ref(&mut self) {
        self.godot_ref_count -= 1;
        out!(
            "  | Storage::on_dec_ref (rc={})     <{}> -- {:?}",
            self.godot_ref_count,
            type_name::<T>(),
            self.user_instance
        );
    }

    /* pub fn destroy(&mut self) {
        assert!(
            self.user_instance.is_some(),
            "Cannot destroy user instance which is not yet initialized"
        );
        assert!(
            !self.destroyed,
            "Cannot destroy user instance multiple times"
        );
        self.user_instance = None; // drops T
                                   // TODO drop entire Storage
    }*/

    #[must_use]
    pub fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    pub fn get(&self) -> cell::Ref<T> {
        self.user_instance.borrow()
    }

    pub fn get_mut(&mut self) -> cell::RefMut<T> {
        self.user_instance.borrow_mut()
    }

    pub fn mark_destroyed_by_godot(&mut self) {
        out!(
            "    Storage::mark_destroyed_by_godot -- {:?}",
            self.user_instance
        );
        self.lifecycle = Lifecycle::Destroying;
        out!(
            "    mark;  self={:?}, val={:?}",
            self as *mut _,
            self.lifecycle
        );
    }

    #[inline(always)]
    pub fn destroyed_by_godot(&self) -> bool {
        out!(
            "    is_d;  self={:?}, val={:?}",
            self as *const _,
            self.lifecycle
        );
        matches!(self.lifecycle, Lifecycle::Destroying | Lifecycle::Dead)
    }
}

impl<T: GodotClass> Drop for InstanceStorage<T> {
    fn drop(&mut self) {
        out!(
            "    Storage::drop (rc={})           <{}> -- {:?}",
            self.godot_ref_count,
            type_name::<T>(),
            self.user_instance
        );
        //let _ = mem::take(&mut self.user_instance);
        out!(
            "    Storage::drop end              <{}>  -- {:?}",
            type_name::<T>(),
            self.user_instance
        );
    }
}

/// Interprets the opaque pointer as pointing to `InstanceStorage<T>`.
///
/// Note: returns reference with unbounded lifetime; intended for local usage
// FIXME unbounded ref AND &mut out of thin air is a huge hazard -- consider using with_storage(ptr, closure) and drop_storage(ptr)
pub unsafe fn as_storage<'u, T: GodotClass>(
    instance_ptr: sys::GDExtensionClassInstancePtr,
) -> &'u mut InstanceStorage<T> {
    &mut *(instance_ptr as *mut InstanceStorage<T>)
}

pub fn nop_instance_callbacks() -> sys::GDNativeInstanceBindingCallbacks {
    // These could also be null pointers, if they are definitely not invoked (e.g. create_callback only passed to object_get_instance_binding(),
    // when there is already a binding). Current "empty but not null" impl corresponds to godot-cpp (wrapped.hpp).
    sys::GDNativeInstanceBindingCallbacks {
        create_callback: Some(create_callback),
        free_callback: Some(free_callback),
        reference_callback: Some(reference_callback),
    }
}

extern "C" fn create_callback(
    _p_token: *mut std::os::raw::c_void,
    _p_instance: *mut std::os::raw::c_void,
) -> *mut std::os::raw::c_void {
    // There is no "instance binding" for Godot types like Node3D -- this would be the user-defined Rust class
    std::ptr::null_mut()
}

extern "C" fn free_callback(
    _p_token: *mut std::os::raw::c_void,
    _p_instance: *mut std::os::raw::c_void,
    _p_binding: *mut std::os::raw::c_void,
) {
}

extern "C" fn reference_callback(
    _p_token: *mut std::os::raw::c_void,
    _p_binding: *mut std::os::raw::c_void,
    _p_reference: sys::GDNativeBool,
) -> sys::GDNativeBool {
    true as u8
}
