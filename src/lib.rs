use std::sync::Mutex;
use std::ptr;
use std::ptr::NonNull;

pub struct TsQueue<T> {
    head: Mutex<NonNull<Node<T>>>,
    tail: Mutex<NonNull<Node<T>>>
}
impl<T> Drop for TsQueue<T> {
    fn drop(&mut self) {
        let mut x = unsafe {Box::<Node<T>>::from_raw(self.head.get_mut().unwrap().as_ptr())};
        while let Some(next) = x.next.take() {
            x = unsafe {Box::from_raw(next.as_ptr())};
        }
    }
}

unsafe impl<T: Send> Send for TsQueue<T> {}
unsafe impl<T: Send> Sync for TsQueue<T> {}


struct Node<T> {
    data: Option<T>,
    next: Option<NonNull<Node<T>>>
}
impl<T> Drop for Node<T> {
    fn drop(&mut self) {
        unsafe { self.next.map_or((),|n| {Box::from_raw(n.as_ptr());})}
    }
}

impl<T> Node<T> {
    fn new() -> NonNull<Self<>> {
        Box::leak(Box::new(Self {
            data: None,
            next: None,
        })).into()
    }
}

impl<T> TsQueue<T> {
    pub fn new() -> Self {
        let dummy = Node::new();
        let tail = Mutex::new(dummy);
        let head = Mutex::new(dummy);
        Self {
            head,
            tail
        }
    }

    pub fn enqueue(&self, data: T) {
        let node = Node::new();
        let new_tail = node;
        let mut tail = self.tail.lock().expect("Unable to lock tail mutex");
        unsafe {
            tail.as_mut().data = Some(data);
            tail.as_mut().next = Some(node);
        }
        *tail = new_tail;
    }

    pub fn dequeue(&self) -> Option<T> {
        let mut head = self.head.lock().expect("Unable to lock head");
        if ptr::eq(head.as_ptr(), self.get_tail_ptr()) {
            return None;
        }
        let mut head_box = unsafe{Box::<Node<T>>::from_raw(head.as_ptr())};
        let data = head_box.data.take();
        let new_head = head_box.next.take().expect("head != tail but head.next is empty");
        *head = new_head;
        data
    }

    fn get_tail_ptr(&self) -> *const Node<T> {
        self.tail.lock().expect("Unable to lock tail").as_ptr()
    }
}


#[cfg(test)]
mod tests {
    use crate::TsQueue;

    #[test]
    fn single_threaded() {
        let queue: TsQueue<i32> = TsQueue::new();
        let data_expected: Vec<_> = (0..20).into_iter().collect();
        let mut data = data_expected.clone();
        queue.enqueue(1);
        queue.dequeue();
        for i in data.drain(..) {
            queue.enqueue(i);
        }
        while let Some(i) = queue.dequeue() {
            data.push(i);
        }
        assert_eq!(data_expected, data);
    }

    #[test]
    fn multi_threaded() {
        let queue = TsQueue::new();
        let data_expected: Vec<_> = (0..=9999).into_iter().collect();
        let mut data_recv = Vec::with_capacity(10000);


        rayon::join(
            || {
                for i in &data_expected {
                    queue.enqueue(*i);
                }
            },
            || {
                loop {
                    if let Some(i) = queue.dequeue() {
                        data_recv.push(i);
                        if i == 9999 {
                            break;
                        }
                    }
                }
            }
        );

        assert_eq!(data_expected, data_recv);
    }
}
