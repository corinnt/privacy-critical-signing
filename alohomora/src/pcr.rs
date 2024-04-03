// Creation of this must be signed.
#[derive(Clone, Copy, Debug)]
pub struct PrivacyCriticalRegion<F> {
    f: F,
}
impl<F> PrivacyCriticalRegion<F> {
    pub const fn new(f: F, _author: Signature, _reviewer: Signature) -> Self {
        PrivacyCriticalRegion { f }
    }
    pub fn get_functor(self) -> F {
        self.f
    }
}

pub struct Signature {
    pub username: &'static str,
    pub signature: &'static str,
}