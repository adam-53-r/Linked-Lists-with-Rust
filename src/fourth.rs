use std::rc::Rc;
use std::cell::{Ref, RefCell, RefMut};

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

pub struct Node<T> {
    elem: T,
    next: Link<T>,
    prev: Link<T>
}

impl<T> Node<T> {
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            elem,
            next: None,
            prev: None
        }))
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None, tail: None }
    }

    pub fn push_front(&mut self, elem: T) {
        let new_head = Node::new(elem);
        match self.head.take() {
            Some(old_head) => {
                // non-empty list, we have to connect to old_head
                old_head.borrow_mut().prev = Some(new_head.clone()); // +1 new_head reference
                new_head.borrow_mut().next = Some(old_head);         // +1 old_head reference
                self.head = Some(new_head);             // +1 new_head reference -1 old_head ref
                // total +2 new_head, +0 old_head
            },
            None => {
                // empty list, need to set the tail
                self.tail = Some(new_head.clone()); // +1 new_head reference
                self.head = Some(new_head);         // +1 new_head reference
                // total +2 new_head
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        // need to take old_head ensuring 2 references to old_head
        // get discarded and no additional reference to new_head is created or discarded (+0)
        self.head.take().map(|old_head| {   // -1 old_head ref from self.head
            match old_head.borrow_mut().next.take() {            // -1 new_head ref from old_head.next
                Some(new_head) => {
                    // not emptying list
                    new_head.borrow_mut().prev.take();           // -1 old_head ref from new_head.prev
                    self.head = Some(new_head);                  // +1 new_head ref to self.head
                    // total: -2 old_head ref, +0 new_head ref
                },
                None => {
                    // emptying list
                    self.tail.take();
                    // total: -2 old_head
                }
            }
            Rc::try_unwrap(old_head).ok().unwrap().into_inner().elem
        })
    }

    pub fn peek_front(&self) -> Option<Ref<T>> {
        // map consumes self, so we use as_ref()
        self.head.as_ref().map(|node| {
            Ref::map(node.borrow(), |node| &node.elem)
        })
    }

    pub fn peek_front_mut(&mut self) -> Option<RefMut<T>> {
        // map consumes self, so we use as_ref()
        self.head.as_ref().map(|node| {
            RefMut::map(node.borrow_mut(), |node| &mut node.elem)
        })
    }

    pub fn push_back(&mut self, elem: T) {
        let new_tail = Node::new(elem);
        match self.tail.take() {
            Some(old_tail) => {
                // non-empty list, we have to connect to old_tail
                old_tail.borrow_mut().next = Some(new_tail.clone()); // +1 new_tail reference
                new_tail.borrow_mut().prev = Some(old_tail);         // +1 old_tail reference
                self.tail = Some(new_tail);             // +1 new_tail reference -1 old_tail ref
                // total +2 new_tail, +0 old_tail
            }
            None => {
                // empty list, need to set the head
                self.tail = Some(new_tail.clone()); // +1 new_tail reference
                self.head = Some(new_tail);         // +1 new_tail reference
                // total +2 new_tail
            }
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        // need to take old_tail ensuring 2 references to old_tail
        // get discarded and no additional reference to new_tail is created or discarded (+0)
        self.tail.take().map(|old_tail| {   // -1 old_tail ref from self.tail
            match old_tail.borrow_mut().prev.take() {            // -1 new_tail ref from old_tail.prev
                Some(new_tail) => {
                    // not emptying list
                    new_tail.borrow_mut().next.take();           // -1 old_tail ref from new_tail.next
                    self.tail = Some(new_tail);                  // +1 new_tail ref to self.tail
                    // total: -2 old_tail ref, +0 new_tail ref
                }
                None => {
                    // emptying list
                    self.head.take();
                    // total: -2 old_tail
                }
            }
            Rc::try_unwrap(old_tail).ok().unwrap().into_inner().elem
        })
    }

    pub fn peek_back(&self) -> Option<Ref<T>> {
        // map consumes self, so we use as_ref()
        self.tail.as_ref().map(|node| {
            Ref::map(node.borrow(), |node| &node.elem)
        })
    }

    pub fn peek_back_mut(&mut self) -> Option<RefMut<T>> {
        // map consumes self, so we use as_ref()
        self.tail.as_ref().map(|node| {
            RefMut::map(node.borrow_mut(), |node| &mut node.elem)
        })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

pub struct IntoIter<T>(List<T>);

impl<T> List<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}

#[cfg(test)]
mod test {
    use std::ops::DerefMut;
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop_front(), None);

        // Populate list
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        list.push_back(4);
        list.push_back(5);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_back(), Some(5));
        assert_eq!(list.pop_back(), Some(4));

        // Push some more just to make sure nothing's corrupted
        list.push_front(4);
        list.push_front(5);
        list.push_back(6);
        list.push_back(7);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));
        assert_eq!(list.pop_back(), Some(7));
        assert_eq!(list.pop_back(), Some(6));

        // Check exhaustion
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert!(list.peek_front().is_none());
        assert!(list.peek_back().is_none());
        list.push_front(1); list.push_front(2); list.push_front(3);
        assert_eq!(&*list.peek_front().unwrap(), &3);
        assert_eq!(&*list.peek_back().unwrap(), &1);
    }

    #[test]
    fn peek_mut() {
        let mut list = List::new();
        list.push_back(1); list.push_back(2); list.push_back(3);
        assert_eq!(&mut *list.peek_back_mut().unwrap(), &mut 3);
        *&mut *list.peek_back_mut().unwrap() = 10;
        assert_eq!(list.peek_back_mut().unwrap().deref_mut(), &mut 10);
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push_back(1); list.push_back(2); list.push_back(3);
        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), None);
    }
}
