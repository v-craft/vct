use core::{cell::RefCell, ops::DerefMut};
use alloc::vec::Vec;
use thread_local::ThreadLocal;

pub struct Parallel<T: Send> {
    locals: ThreadLocal<RefCell<T>>,
}

impl<T: Send> Default for Parallel<T> {
    fn default() -> Self {
        Self {
            locals: ThreadLocal::default(),
        }
    }
}

impl<T: Send> Parallel<T> {
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &'_ mut T> {
        self.locals.iter_mut().map(RefCell::get_mut)
    }

    pub fn clear(&mut self) {
        self.locals.clear();
    }

    pub fn scope_or<R>(&self, create: impl FnOnce() -> T, f: impl FnOnce(&mut T) -> R) -> R {
        f(&mut self.borrow_local_mut_or(create))
    }

    pub fn borrow_local_mut_or(
        &self,
        create: impl FnOnce() -> T,
    ) -> impl DerefMut<Target = T> + '_ {
        self.locals.get_or(|| RefCell::new(create())).borrow_mut()
    }
}

impl<T: Default + Send> Parallel<T> {
    pub fn scope<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.scope_or(Default::default, f)
    }

    pub fn borrow_local_mut(&self) -> impl DerefMut<Target = T> + '_ {
        self.borrow_local_mut_or(Default::default)
    }
}

impl<T, I> Parallel<I>
where
    I: IntoIterator<Item = T> + Default + Send + 'static,
{
    pub fn drain(&mut self) -> impl Iterator<Item = T> + '_ {
        self.locals.iter_mut().flat_map(|item| item.take())
    }
}

impl<T: Send> Parallel<Vec<T>> {
    pub fn drain_into(&mut self, out: &mut Vec<T>) {
        let size = self
            .locals
            .iter_mut()
            .map(|queue| queue.get_mut().len())
            .sum();
        out.reserve(size);
        for queue in self.locals.iter_mut() {
            out.append(queue.get_mut());
        }
    }
}

