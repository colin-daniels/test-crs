use crate::engine::Value;

// pub fn lowercase(var: &Variable) -> Variable {
//     var.value
//     todo!()
// }

// use super::Variable;
// use std::cell::RefCell;
// use std::iter::Map;
//
//
// pub struct Transform<'a, I, F> {
//     iter: RefCell<Map<I, F>>,
//     cache: RefCell<Vec<Variable<'a>>>,
// }
//
// impl<'a, I, F> Transform<'a, I, F>
// where
//     I: Iterator<Item = Variable<'a>>,
//     F: FnMut(I::Item) -> B,
// {
//     #[inline]
//     pub fn cache_size(&self) -> usize {
//         // Note: This is safe because we know Vec::len is essentially an atomic operation
//         // in that it will always return a valid value in this case.
//         unsafe { self.cache.try_borrow_unguarded() }.unwrap().len()
//     }
//
//     #[inline]
//     pub fn next(&self) -> Option<Variable<'a>> {
//         if let Some(next) = self.iter.borrow_mut().next() {
//             self.cache.borrow_mut().push(next);
//             Some(next)
//         } else {
//             None
//         }
//     }
//
//     pub fn iter(&self) -> Iter<'a, '_, I, F> {
//         Iter {
//             index: 0,
//             transform: self,
//         }
//     }
// }
//
// pub struct Iter<'a, 'b, I, F> {
//     index: usize,
//     transform: &'b Transform<'a, I, F>,
// }
//
// impl<'a, 'b, I, F> Iterator for Iter<'a, 'b, I, F>
// where
//     I: Iterator<Item = Variable<'a>>,
//     F: FnMut(I::Item) -> B,
// {
//     type Item = Variable<'a>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         let index = self.index;
//
//         if index < self.transform.cache_size() {
//             // current index is less than the total number of cache entries,
//             // so we can just return the cached value
//             let cache = self.transform.cache.borrow();
//             self.index += 1;
//             Some(cache[index])
//         } else {
//             if let Some(next) = self.transform.next() {
//                 self.index += 1;
//                 Some(next)
//             } else {
//                 None
//             }
//         }
//     }
// }
