pub trait WithProcedure {
    fn with<F>(self, f: F) -> Self
    where
        F: (FnOnce(Self) -> Self) + 'static,
        Self: Sized;
}

pub trait WithMutProcedure {
    fn with_mut<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut Self) + 'static;
}

impl<T: Sized> WithProcedure for T {
    fn with<F>(self, f: F) -> Self
    where
        F: (FnOnce(Self) -> Self) + 'static,
    {
        f(self)
    }
}

impl<T> WithMutProcedure for T {
    fn with_mut<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Self) + 'static,
    {
        f(&mut self);
        self
    }
}