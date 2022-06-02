use core::{mem::MaybeUninit, pin::Pin};

pub unsafe trait ___PlaceInit {
    type Init;
    type Raw;

    unsafe fn ___init(self) -> Self::Init;

    unsafe fn ___as_mut_ptr(&mut self, _proof: impl FnOnce(Self::Raw)) -> *mut Self::Raw;
}

pub unsafe trait ___PinnedPlace {}

unsafe impl<T> ___PlaceInit for MaybeUninit<T> {
    type Init = T;
    type Raw = T;

    unsafe fn ___init(self) -> Self::Init {
        self.assume_init()
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: impl FnOnce(Self::Raw)) -> *mut Self::Raw {
        self.as_mut_ptr()
    }
}

unsafe impl<T> ___PlaceInit for Box<MaybeUninit<T>> {
    type Init = Box<T>;
    type Raw = T;

    unsafe fn ___init(self) -> Self::Init {
        self.assume_init()
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: impl FnOnce(Self::Raw)) -> *mut Self::Raw {
        MaybeUninit::as_mut_ptr(&mut **self)
    }
}

unsafe impl<P, T> ___PlaceInit for Pin<P>
where
    P: ___PlaceInit + core::ops::DerefMut<Target = T>,
    P::Init: core::ops::Deref,
    T: ___PlaceInit<Raw = P::Raw>,
{
    type Init = Pin<P::Init>;
    type Raw = P::Raw;

    unsafe fn ___init(self) -> Self::Init {
        Pin::new_unchecked(Pin::into_inner_unchecked(self).___init())
    }

    unsafe fn ___as_mut_ptr(&mut self, _proof: impl FnOnce(Self::Raw)) -> *mut Self::Raw {
        Pin::get_unchecked_mut(self.as_mut()).___as_mut_ptr(_proof)
    }
}

unsafe impl<P, T> ___PinnedPlace for Pin<P>
where
    P: ___PlaceInit + core::ops::DerefMut<Target = T>,
    P::Init: core::ops::Deref,
    T: ___PlaceInit<Raw = P::Raw>,
{
}

pub unsafe trait ___PinData {
    type ___PinData;
}
