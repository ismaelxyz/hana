//! Basic implementation of a mark and sweep garbage collector

pub use libc::c_void;
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::cmp::Ordering;
use std::ptr::{drop_in_place, null_mut, NonNull};

use super::value::Value;
use super::vm::Vm;

#[derive(Debug, PartialEq)]
enum GcNodeColor {
    White,
    Gray,
    Black,
}

// node
pub struct GcNode {
    next: *mut GcNode,
    size: usize,
    color: GcNodeColor,
    native_refs: usize,
    // tracer gets called on the marking phase
    tracer: GenericTraceFunction,
    /* finalizer gets called with a pointer to
     * the data that's about to be freed */
    finalizer: GenericFunction,
}

impl GcNode {
    pub fn alloc_size<T: Sized>() -> usize {
        // number of bytes needed to allocate node for <T>
        use std::mem::size_of;
        size_of::<GcNode>() + size_of::<T>()
    }
}

type GenericFunction = unsafe fn(*mut c_void);
// a generic function that takes in some pointer
// this might be a finalizer or a tracer function
// TODO maybe replace this with Any

// manager
const INITIAL_THRESHOLD: usize = 4096;
const USED_SPACE_RATIO: f64 = 0.7;
pub struct GcManager {
    first_node: *mut GcNode,
    last_node: *mut GcNode,
    bytes_allocated: usize,
    gray_nodes: Vec<*mut GcNode>,
    threshold: usize,
    enabled: bool,
}

impl GcManager {
    pub fn new() -> GcManager {
        GcManager {
            first_node: null_mut(),
            last_node: null_mut(),
            bytes_allocated: 0,
            gray_nodes: Vec::new(),
            threshold: INITIAL_THRESHOLD,
            enabled: false,
        }
    }

    unsafe fn malloc_raw<T: Sized + GcTraceable>(
        &mut self,
        vm: &Vm,
        x: T,
        finalizer: GenericFunction,
    ) -> *mut T {
        let size = GcNode::alloc_size::<T>();
        let node: *mut GcNode = self
            .cycle(vm, size)
            .unwrap_or_else(|| {
                let layout = Layout::from_size_align(size, 2).unwrap();
                NonNull::new(alloc_zeroed(layout) as *mut GcNode).unwrap()
            })
            .as_ptr();
        // append node
        if self.first_node.is_null() {
            self.first_node = node;
            self.last_node = node;
            (*node).next = null_mut();
        } else {
            (*self.last_node).next = node;
            (*node).next = null_mut();
            self.last_node = node;
        }
        (*node).native_refs = 1;
        (*node).tracer = std::mem::transmute::<
            *mut libc::c_void,
            unsafe fn(*mut libc::c_void, *mut libc::c_void),
        >(T::trace as *mut c_void);
        (*node).finalizer = finalizer;
        (*node).size = size;
        self.bytes_allocated += (*node).size;
        // gray out the node
        // TODO: we currently move the write barrier forward rather than backwards
        // this probably is less efficient than setting the newly allocated node
        // to white then resetting its soon-to-be parent to gray (for retracing)
        (*node).color = GcNodeColor::Gray;
        self.gray_nodes.push(node);
        // return the body aka (start byte + sizeof(GCNode))
        std::mem::forget(std::mem::replace(&mut *(node.add(1) as *mut T), x));
        node.add(1) as *mut T
    }

    pub fn malloc<T: Sized + GcTraceable>(&mut self, vm: &Vm, val: T) -> Gc<T> {
        Gc {
            ptr: NonNull::new(unsafe {
                self.malloc_raw(vm, val, |ptr| drop_in_place::<T>(ptr as *mut T))
            })
            .unwrap(),
        }
    }

    pub unsafe fn push_gray_body(&mut self, ptr: *mut c_void) {
        push_gray_body(&mut self.gray_nodes, ptr)
    }

    // state
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    #[allow(dead_code)]
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    // gc algorithm
    unsafe fn cycle(&mut self, vm: &Vm, size: usize) -> Option<NonNull<GcNode>> {
        if !self.enabled || self.bytes_allocated < self.threshold {
            return None;
        }
        // marking phase
        let gray_nodes = std::mem::take(&mut self.gray_nodes);
        for node in gray_nodes.iter() {
            let body = node.add(1) as *mut c_void;
            (**node).color = GcNodeColor::Black;

            let tracer_fn = (**node).tracer;
            let mut_ptr =
                &mut self.gray_nodes as *mut std::vec::Vec<*mut GcNode> as *mut libc::c_void;

            tracer_fn(body, mut_ptr);
        }
        // nothing left to traverse, sweeping phase:
        if self.gray_nodes.is_empty() {
            let mut prev: *mut GcNode = null_mut();
            // sweep
            let mut node = self.first_node;
            let mut first_fitting_node: Option<NonNull<GcNode>> = None;
            while !node.is_null() {
                let next: *mut GcNode = (*node).next;
                let mut freed = false;
                if (*node).native_refs == 0 && (*node).color == GcNodeColor::White {
                    freed = true;
                    let body = node.add(1);

                    // remove from ll
                    if prev.is_null() {
                        self.first_node = (*node).next;
                    } else {
                        (*prev).next = (*node).next;
                    }
                    if (*node).next.is_null() {
                        self.last_node = prev;
                    }
                    self.bytes_allocated -= (*node).size;

                    // call finalizer
                    let finalizer = (*node).finalizer;
                    finalizer(body as *mut c_void);

                    // if this node fits then record it
                    if (*node).size == size && first_fitting_node.is_none() {
                        std::ptr::write_bytes(node as *mut u8, 0, (*node).size);
                        first_fitting_node = Some(NonNull::new_unchecked(node));
                    } else {
                        // else just free it
                        let layout = Layout::from_size_align((*node).size, 2).unwrap();
                        dealloc(node as *mut u8, layout);
                    }
                } else if (*node).native_refs != 0 {
                    self.gray_nodes.push(node);
                } else {
                    (*node).color = GcNodeColor::White;
                }
                if !freed {
                    prev = node;
                }
                node = next;
            }
            vm.trace(&mut self.gray_nodes);

            // we didn't collect enough, grow the ratio
            if ((self.bytes_allocated as f64) / (self.threshold as f64)) > USED_SPACE_RATIO {
                self.threshold = (self.bytes_allocated as f64 / USED_SPACE_RATIO) as usize;
            }

            // return first fitting node if there is any
            first_fitting_node
        } else {
            None
        }
    }
}

impl std::ops::Drop for GcManager {
    fn drop(&mut self) {
        unsafe {
            let mut node: *mut GcNode = self.first_node;
            while !node.is_null() {
                let next: *mut GcNode = (*node).next;
                let body = node.add(1);
                // call finalizer
                let finalizer = (*node).finalizer;
                finalizer(body as *mut c_void);
                // free memory
                let layout = Layout::from_size_align((*node).size, 2).unwrap();
                dealloc(node as *mut u8, layout);
                node = next;
            }
        }
    }
}

// #region gc struct
#[repr(transparent)]
pub struct Gc<T: Sized + GcTraceable> {
    ptr: NonNull<T>,
}

impl<T: Sized + GcTraceable> Gc<T> {
    // ptrs
    pub fn to_raw(&self) -> *const T {
        self.ptr.as_ptr()
    }
    pub unsafe fn into_raw(self) -> *mut T {
        self.ptr.as_ptr()
    }

    // refs with interior mutability
    pub fn inner_mut_ptr(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: Sized + GcTraceable> std::ops::Drop for Gc<T> {
    fn drop(&mut self) {
        unsafe {
            ref_dec(self.ptr.as_ptr() as *mut libc::c_void);
        }
    }
}

impl<T: Sized + GcTraceable> std::convert::AsRef<T> for Gc<T> {
    fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: Sized + GcTraceable> std::clone::Clone for Gc<T> {
    fn clone(&self) -> Self {
        Gc {
            ptr: unsafe {
                ref_inc(self.ptr.as_ptr() as *mut libc::c_void);
                self.ptr
            },
        }
    }
}

impl<T: Sized + GcTraceable> std::cmp::PartialEq for Gc<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.ptr.as_ptr(), other.ptr.as_ptr())
    }
}

impl<T: Sized + GcTraceable> PartialOrd for Gc<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Sized + GcTraceable> Ord for Gc<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_size = std::mem::size_of_val(self);
        let other_size = std::mem::size_of_val(other);

        self_size.cmp(&other_size)
    }
}

impl<T: Sized + GcTraceable> std::cmp::Eq for Gc<T> {}
// #endregion

// #region traceable
pub trait GcTraceable {
    unsafe fn trace(&self, manager: &mut Vec<*mut GcNode>);
}

type GenericTraceFunction = unsafe fn(*mut c_void, *mut c_void);

// native traceables
impl GcTraceable for Vec<Value> {
    unsafe fn trace(&self, gray_nodes: &mut Vec<*mut GcNode>) {
        for val in self.iter() {
            if let Some(ptr) = val.as_gc_pointer() {
                push_gray_body(gray_nodes, ptr);
            }
        }
    }
}
// #endregion

/// # Safety
///
/// a pointer is being modified
pub unsafe fn ref_inc(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let node: *mut GcNode = (ptr as *mut GcNode).sub(1);
    (*node).native_refs += 1;
}

/// # Safety
///
/// a pointer is being modified
pub unsafe fn ref_dec(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let node: *mut GcNode = (ptr as *mut GcNode).sub(1);
    (*node).native_refs -= 1;
}

pub unsafe fn push_gray_body(gray_nodes: &mut Vec<*mut GcNode>, ptr: *mut c_void) {
    let node: *mut GcNode = (ptr as *mut GcNode).sub(1);
    //eprintln!("node: {:p}", node);
    if (*node).color == GcNodeColor::Black || (*node).color == GcNodeColor::Gray {
        return;
    }
    (*node).color = GcNodeColor::Gray;
    gray_nodes.push(node);
}
