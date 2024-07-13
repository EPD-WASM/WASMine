use core::slice;
use std::pin::Pin;

#[derive(Debug)]
pub(crate) struct SegmentedList<T> {
    #[allow(clippy::box_collection)]
    segments: Vec<Pin<Box<Vec<T>>>>,
}

impl<T> SegmentedList<T> {
    pub(crate) fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, segment: T) {
        self.segments.push(Box::pin(vec![segment]));
    }

    pub(crate) fn extend(&mut self, segments: Vec<T>) {
        self.segments.push(Box::pin(segments));
    }

    pub(crate) fn len(&self) -> usize {
        self.segments.iter().map(|s| s.len()).sum()
    }

    pub(crate) fn get_last_segments_ref<'b>(&self) -> &'b mut [T] {
        unsafe {
            slice::from_raw_parts_mut(
                self.segments.last().unwrap().as_ptr() as *mut T,
                self.segments.last().unwrap().len(),
            )
        }
    }
}

impl<T> Default for SegmentedList<T> {
    fn default() -> Self {
        Self::new()
    }
}
