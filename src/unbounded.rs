pub struct UnboundedRef<T: ?Sized>(*const T);

unsafe impl<T: ?Sized + Sync> Send for UnboundedRef<T> {}

unsafe impl<T: ?Sized + Sync> Sync for UnboundedRef<T> {}

impl<T: ?Sized> UnboundedRef<T> {
    pub fn new(x: &T) -> Self { UnboundedRef(x) }
    pub unsafe fn deref_unbounded(&self) -> &T { &*self.0 }
    pub unsafe fn deref_escape<'a, 'b>(&'a self) -> &'b T { &*self.0 }
}

impl<T: ?Sized> Clone for UnboundedRef<T> {
    fn clone(&self) -> Self { UnboundedRef(self.0) }
}