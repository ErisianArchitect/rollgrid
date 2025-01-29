use crate::{bounds2d::Bounds2D, bounds3d::Bounds3D, error_messages::*};
use std::{mem::ManuallyDrop, ptr::NonNull};

/// An array of type `T`.
/// This is an abstraction over the memory meant to be used in rolling grid
/// implementations. This struct allows for taking values from the buffer without
/// dropping the old value, as well as the ability to drop values in place. This
/// gives the user the ability to manually manage dropping of individual regions.
/// The user manages the dimensionality and bounds of the [FixedArray].
pub struct FixedArray<T> {
    pub(crate) ptr: Option<NonNull<T>>,
    pub(crate) capacity: usize,
}

impl<T> FixedArray<T> {
    #[inline(always)]
    fn prealloc_2d(size: (u32, u32), offset: (i32, i32)) -> (NonNull<T>, Bounds2D, usize) {
        let (width, height) = (size.0 as usize, size.1 as usize);
        let x_max = offset.0 as i64 + width as i64;
        if x_max > i32::MAX as i64 {
            panic!("{}", X_MAX_EXCEEDS_MAXIMUM);
        }
        let y_max = offset.1 as i64 + height as i64;
        if y_max > i32::MAX as i64 {
            panic!("{}", Y_MAX_EXCEEDS_MAXIMUM);
        }
        let area = width.checked_mul(height).expect(SIZE_TOO_LARGE);
        if area == 0 {
            panic!("{}", AREA_IS_ZERO);
        }
        unsafe {
            let layout = Self::make_layout(area).expect("Failed to create layout.");
            (
                NonNull::new(std::alloc::alloc(layout) as *mut T).expect("Null pointer."),
                Bounds2D::new(offset, (x_max as i32, y_max as i32)),
                area,
            )
        }
    }

    #[inline(always)]
    fn prealloc_3d(
        size: (u32, u32, u32),
        offset: (i32, i32, i32),
    ) -> (NonNull<T>, Bounds3D, usize) {
        let (width, height, depth) = (size.0 as usize, size.1 as usize, size.2 as usize);
        let x_max = offset.0 as i64 + width as i64;
        if x_max > i32::MAX as i64 {
            panic!("{}", X_MAX_EXCEEDS_MAXIMUM);
        }
        let y_max = offset.1 as i64 + height as i64;
        if y_max > i32::MAX as i64 {
            panic!("{}", Y_MAX_EXCEEDS_MAXIMUM);
        }
        let z_max = offset.2 as i64 + depth as i64;
        if z_max > i32::MAX as i64 {
            panic!("{}", Z_MAX_EXCEEDS_MAXIMUM);
        }
        let volume = width
            .checked_mul(height)
            .expect(SIZE_TOO_LARGE)
            .checked_mul(depth)
            .expect(SIZE_TOO_LARGE);
        if volume == 0 {
            panic!("{}", VOLUME_IS_ZERO);
        }
        unsafe {
            let layout = Self::make_layout(volume).expect("Failed to create layout.");
            (
                NonNull::new(std::alloc::alloc(layout) as *mut T).expect("Null pointer."),
                Bounds3D::new(
                    offset,
                    (
                        x_max as i32,
                        y_max as i32,
                        z_max as i32,
                    ),
                ),
                volume,
            )
        }
    }

    /// Allocate a new [FixedArray] from a 2D size and offset with an
    /// initialization function.
    ///
    /// Initialization happens in the order `x -> y`, that your results will be ordered
    /// like so:
    /// * `(0, 0)`
    /// * `(1, 0)`
    /// * `(0, 1)`
    /// * `(1, 1)`
    pub fn new_2d<F: FnMut((i32, i32)) -> T>(
        size: (u32, u32),
        offset: (i32, i32),
        mut init: F,
    ) -> Self {
        let (ptr, bounds, capacity) = Self::prealloc_2d(size, offset);
        bounds.iter().enumerate().for_each(move |(i, pos)| unsafe {
            let item = ptr.add(i);
            std::ptr::write(item.as_ptr(), init(pos));
        });
        Self {
            ptr: Some(ptr),
            capacity,
        }
    }

    /// Attempt to allocate a new [FixedArray] from a 2D size and offset
    /// with an initialization function.
    ///
    /// Initialization happens in the order `x -> y`, that your results will be ordered
    /// like so:
    /// * `(0, 0)`
    /// * `(1, 0)`
    /// * `(0, 1)`
    /// * `(1, 1)`
    pub fn try_new_2d<E, F: FnMut((i32, i32)) -> Result<T, E>>(
        size: (u32, u32),
        offset: (i32, i32),
        mut init: F,
    ) -> Result<Self, E> {
        let (ptr, bounds, capacity) = Self::prealloc_2d(size, offset);
        bounds.iter().enumerate().try_for_each(move |(i, pos)| {
            unsafe {
                let item = ptr.add(i);
                std::ptr::write(item.as_ptr(), init(pos)?);
            }
            Ok(())
        })?;
        Ok(Self {
            ptr: Some(ptr),
            capacity,
        })
    }

    /// Allocate a new [FixedArray] from a 3D size and offset with an
    /// initialization function.
    ///
    /// Initialization happens in the order `x -> z -> y`, that your results
    /// will be ordered like so:
    /// * `(0, 0, 0)`
    /// * `(1, 0, 0)`
    /// * `(0, 0, 1)`
    /// * `(1, 0, 1)`
    /// * `(0, 1, 0)`
    /// * `(1, 1, 0)`
    /// * `(0, 1, 1)`
    /// * `(1, 1, 1)`
    pub fn new_3d<F: FnMut((i32, i32, i32)) -> T>(
        size: (u32, u32, u32),
        offset: (i32, i32, i32),
        mut init: F,
    ) -> Self {
        let (ptr, bounds, capacity) = Self::prealloc_3d(size, offset);
        bounds.iter().enumerate().for_each(move |(i, pos)| unsafe {
            let item = ptr.add(i);
            std::ptr::write(item.as_ptr(), init(pos));
        });
        Self {
            ptr: Some(ptr),
            capacity,
        }
    }

    /// Attempt to allocate a new [FixedArray] from a 3D size and offset
    /// with an initialization function.
    ///
    /// Initialization happens in the order `x -> z -> y`, that your results
    /// will be ordered like so:
    /// * `(0, 0, 0)`
    /// * `(1, 0, 0)`
    /// * `(0, 0, 1)`
    /// * `(1, 0, 1)`
    /// * `(0, 1, 0)`
    /// * `(1, 1, 0)`
    /// * `(0, 1, 1)`
    /// * `(1, 1, 1)`
    pub fn try_new_3d<E, F: FnMut((i32, i32, i32)) -> Result<T, E>>(
        size: (u32, u32, u32),
        offset: (i32, i32, i32),
        mut init: F,
    ) -> Result<Self, E> {
        let (ptr, bounds, capacity) = Self::prealloc_3d(size, offset);
        bounds.iter().enumerate().try_for_each(move |(i, pos)| {
            unsafe {
                let item = ptr.add(i);
                std::ptr::write(item.as_ptr(), init(pos)?);
            }
            Ok(())
        })?;
        Ok(Self {
            ptr: Some(ptr),
            capacity,
        })
    }

    /// Deallocates the internal buffer in this [FixedArray].
    pub unsafe fn dealloc(&mut self) {
        self.internal_dealloc(true);
        self.capacity = 0;
    }

    /// Set `drop` to `false` if you have already manually dropped the items.
    pub(crate) unsafe fn internal_dealloc(&mut self, drop: bool) {
        if let Some(ptr) = self.ptr.take() {
            unsafe {
                if std::mem::needs_drop::<T>() && drop {
                    (0..self.capacity).map(|i| ptr.add(i)).for_each(|mut item| {
                        std::ptr::drop_in_place(item.as_mut());
                    });
                }
                let layout = self.layout();
                std::alloc::dealloc(ptr.as_ptr() as *mut u8, layout);
            }
        }
    }

    /// Deallocates the buffer and forgets about the contained items (does not drop them).
    pub(crate) unsafe fn forget_dealloc(&mut self) {
        self.internal_dealloc(false);
    }

    /// Only use this method if you know what you are doing.
    /// It uses [std::ptr::read] to read the value at `index`.
    /// If you use this method, make sure to keep track of which cells are read so that you can manually drop the cells that are not read.
    pub(crate) unsafe fn read(&self, index: usize) -> T {
        std::ptr::read(&self[index])
    }

    /// Only use this method if you know what you are doing.
    /// It uses [std::ptr::write] to write into the slot at `index` without dropping
    /// the inner value.
    /// It is advised to use [FixedArray::read()] or [FixedArray::drop_in_place()] before
    /// calling this method.
    pub(crate) unsafe fn write(&mut self, index: usize, value: T) {
        std::ptr::write(&mut self[index], value);
    }

    /// Replace item at `index` using `replace` function that takes as input the old value and returns the new value.
    /// This will swap the value in-place.
    pub fn replace_with<F: FnOnce(T) -> T>(&mut self, index: usize, replace: F) {
        unsafe {
            std::ptr::write(&mut self[index], replace(std::ptr::read(&self[index])));
        }
    }

    /// Replace item at `index` using [std::mem::replace], returns the old value.
    pub fn replace(&mut self, index: usize, value: T) -> T {
        std::mem::replace(&mut self[index], value)
    }

    /// Drops the value at `index` in place using [std::ptr::drop_in_place].
    pub(crate) unsafe fn drop_in_place(&mut self, index: usize) {
        std::ptr::drop_in_place(&mut self[index]);
    }

    /// Returns the [std::alloc::Layout] associated with this [FixedArray].
    fn layout(&self) -> std::alloc::Layout {
        Self::make_layout(self.capacity).unwrap()
    }

    /// Makes an [std::alloc::Layout] for [FixedArray<T>] with `capacity`.
    fn make_layout(capacity: usize) -> Result<std::alloc::Layout, std::alloc::LayoutError> {
        std::alloc::Layout::array::<T>(capacity)
    }

    /// Gets the length of the array.
    pub fn len(&self) -> usize {
        self.capacity
    }

    /// Returns the array as a slice.
    pub fn as_slice(&self) -> &[T] {
        let Some(ptr) = self.ptr else {
            panic!("{}", NOT_ALLOCATED);
        };
        unsafe { std::slice::from_raw_parts(ptr.as_ref(), self.capacity) }
    }

    /// Returns the array as a mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let Some(mut ptr) = self.ptr else {
            panic!("{}", NOT_ALLOCATED);
        };
        unsafe { std::slice::from_raw_parts_mut(ptr.as_mut(), self.capacity) }
    }

    /// Returns the internal pointer. This may return `null` if the buffer has already been deallocated.
    pub unsafe fn as_ptr(&self) -> *const T {
        self.ptr.map_or(std::ptr::null(), |ptr| ptr.as_ptr())
    }

    /// Returns the internal mutable pointer. This may return `null` if the buffer has already been deallocated.
    pub unsafe fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.map_or(std::ptr::null_mut(), NonNull::as_ptr)
    }

    /// Converts the array into a boxed slice.
    pub fn into_boxed_slice(self) -> Box<[T]> {
        let Some(ptr) = self.ptr else {
            panic!("{}", NOT_ALLOCATED);
        };
        unsafe {
            let slice_ptr = std::ptr::slice_from_raw_parts_mut(ptr.as_ptr(), self.capacity);
            let result = Box::from_raw(slice_ptr);
            std::mem::forget(self);
            result
        }
    }

    /// Converts the array into a `Vec<T>`.
    pub fn into_vec(self) -> Vec<T> {
        let Some(ptr) = self.ptr else {
            panic!("{}", NOT_ALLOCATED);
        };
        unsafe {
            let result = Vec::from_raw_parts(ptr.as_ptr(), self.capacity, self.capacity);
            std::mem::forget(self);
            result
        }
    }

    /// Creates an iterator over elements by reference in the array.
    pub fn iter(&self) -> FixedArrayRefIterator<'_, T> {
        FixedArrayRefIterator {
            array: self,
            index: 0,
        }
    }

    /// Returns the raw pointer and capacity.
    pub unsafe fn into_raw(self) -> (*mut T, usize) {
        let ptr = self
            .ptr
            .map(|ptr| ptr.as_ptr())
            .unwrap_or_else(|| std::ptr::null_mut());
        let capacity = self.capacity;
        (
            ptr,
            capacity
        )
    }

    /// Creates a new FixedArray from a raw pointer and a capacity.
    pub unsafe fn from_raw(data: *mut T, capacity: usize) -> Self {
        if data.is_null() {
            Self {
                ptr: None,
                capacity: 0,
            }
        } else {
            Self {
                ptr: Some(NonNull::new_unchecked(data)),
                capacity,
            }
        }
    }
}

pub struct FixedArrayRefIterator<'a, T> {
    array: &'a FixedArray<T>,
    index: usize,
}

impl<'a, T> Iterator for FixedArrayRefIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.array.capacity {
            return None;
        }
        let result = Some(&self.array[self.index]);
        self.index += 1;
        result
    }
}

impl<T> IntoIterator for FixedArray<T> {
    type IntoIter = FixedArrayIterator<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        FixedArrayIterator {
            array: ManuallyDrop::new(self),
            index: 0,
        }
    }
}

pub struct FixedArrayIterator<T> {
    array: ManuallyDrop<FixedArray<T>>,
    index: usize,
}

impl<T> Iterator for FixedArrayIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.array.capacity {
            return None;
        }
        unsafe {
            let result = Some(self.array.read(self.index));
            self.index += 1;
            result
        }
    }
}

impl<T> Drop for FixedArrayIterator<T> {
    fn drop(&mut self) {
        if std::mem::needs_drop::<T>() {
            let capacity = self.array.capacity;
            let array = &mut self.array;
            (self.index..capacity).for_each(move |index| unsafe {
                array.drop_in_place(index);
            });
        }
        unsafe {
            self.array.internal_dealloc(false);
        }
    }
}

impl<T> From<FixedArray<T>> for Vec<T> {
    fn from(value: FixedArray<T>) -> Self {
        value.into_vec()
    }
}

impl<T> From<FixedArray<T>> for Box<[T]> {
    fn from(value: FixedArray<T>) -> Self {
        value.into_boxed_slice()
    }
}

impl<T> std::ops::Deref for FixedArray<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> std::ops::DerefMut for FixedArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T: Default> FixedArray<T> {
    /// Takes the value at `index` while replacing the old value with [Default::default()].
    pub fn take(&mut self, index: usize) -> T {
        self.replace(index, Default::default())
    }
}

impl<T> std::ops::Index<usize> for FixedArray<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        if let Some(ptr) = self.ptr {
            assert!(index < self.capacity, "Index out of bounds.");
            unsafe { ptr.add(index).as_ref() }
        } else {
            panic!("Unallocated buffer.");
        }
    }
}

impl<T> std::ops::IndexMut<usize> for FixedArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if let Some(ptr) = self.ptr {
            assert!(index < self.capacity, "Index out of bounds.");
            unsafe { ptr.add(index).as_mut() }
        } else {
            panic!("Unallocated buffer.");
        }
    }
}

impl<T> Drop for FixedArray<T> {
    fn drop(&mut self) {
        unsafe {
            self.internal_dealloc(true);
        }
    }
}
