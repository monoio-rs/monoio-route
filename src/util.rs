use crate::{Params, ParamsOwned, ParamsStr, ParamsStrOwned};

pub trait ParamsConvertStr<'k, 'v> {
    /// # Safety
    /// Will assume the params are parsed from pure ascii bytes.
    unsafe fn params_str_unchecked(self) -> ParamsStr<'k, 'v>;
    /// # Safety
    /// Will assume the params are parsed from pure ascii bytes.
    unsafe fn params_str_ref_unchecked(&self) -> &ParamsStr<'k, 'v>;
    /// # Safety
    /// Will assume the params are parsed from pure ascii bytes.
    unsafe fn params_str_mut_unchecked(&mut self) -> &mut ParamsStr<'k, 'v>;
}

impl<'k, 'v> ParamsConvertStr<'k, 'v> for Params<'k, 'v> {
    /// # Safety
    /// Users must ensure the params are parsed from pure ascii bytes.
    #[inline]
    unsafe fn params_str_unchecked(self) -> ParamsStr<'k, 'v> {
        std::mem::transmute(self)
    }
    /// # Safety
    /// Users must ensure the params are parsed from pure ascii bytes.
    #[inline]
    unsafe fn params_str_ref_unchecked(&self) -> &ParamsStr<'k, 'v> {
        std::mem::transmute(self)
    }
    /// # Safety
    /// Users must ensure the params are parsed from pure ascii bytes.
    #[inline]
    unsafe fn params_str_mut_unchecked(&mut self) -> &mut ParamsStr<'k, 'v> {
        std::mem::transmute(self)
    }
}

impl<'k, 'v> ParamsConvertStr<'k, 'v> for ParamsStr<'k, 'v> {
    /// # Safety
    /// Always safe since this method returns itself.
    #[inline]
    unsafe fn params_str_unchecked(self) -> ParamsStr<'k, 'v> {
        self
    }
    /// # Safety
    /// Always safe since this method returns itself.
    #[inline]
    unsafe fn params_str_ref_unchecked(&self) -> &ParamsStr<'k, 'v> {
        self
    }
    /// # Safety
    /// Users must ensure the params are parsed from pure ascii bytes.
    #[inline]
    unsafe fn params_str_mut_unchecked(&mut self) -> &mut ParamsStr<'k, 'v> {
        std::mem::transmute(self)
    }
}

pub trait ParamsGet<T: ?Sized, Q: ?Sized> {
    type Key;
    type Val;
    fn get<'s>(&'s self, key: &'s Q) -> Option<Self::Val>;
    fn get_key_value<'s>(&'s self, key: &'s Q) -> Option<(Self::Key, Self::Val)>;
}

impl<'k, 'v> ParamsGet<[u8], [u8]> for Params<'k, 'v> {
    type Key = &'k [u8];
    type Val = &'v [u8];
    #[inline]
    fn get<'s>(&'s self, key: &'s [u8]) -> Option<&'v [u8]> {
        self.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }
    #[inline]
    fn get_key_value<'s>(&'s self, key: &'s [u8]) -> Option<(&'k [u8], &'v [u8])> {
        self.iter().find(|(k, _)| *k == key).map(|(k, v)| (*k, *v))
    }
}

impl<'k, 'v, const N: usize> ParamsGet<[u8], [u8; N]> for Params<'k, 'v> {
    type Key = &'k [u8];
    type Val = &'v [u8];
    #[inline]
    fn get<'s>(&'s self, key: &'s [u8; N]) -> Option<&'v [u8]> {
        self.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }
    #[inline]
    fn get_key_value<'s>(&'s self, key: &'s [u8; N]) -> Option<(&'k [u8], &'v [u8])> {
        self.iter().find(|(k, _)| *k == key).map(|(k, v)| (*k, *v))
    }
}

impl<'k, 'v> ParamsGet<[u8], str> for Params<'k, 'v> {
    type Key = &'k [u8];
    type Val = &'v [u8];
    #[inline]
    fn get<'s>(&'s self, key: &'s str) -> Option<&'v [u8]> {
        self.iter()
            .find(|(k, _)| *k == key.as_bytes())
            .map(|(_, v)| *v)
    }
    #[inline]
    fn get_key_value<'s>(&'s self, key: &'s str) -> Option<(&'k [u8], &'v [u8])> {
        self.iter()
            .find(|(k, _)| *k == key.as_bytes())
            .map(|(k, v)| (*k, *v))
    }
}

impl<'k, 'v> ParamsGet<str, [u8]> for ParamsStr<'k, 'v> {
    type Key = &'k str;
    type Val = &'v str;
    #[inline]
    fn get<'s>(&'s self, key: &'s [u8]) -> Option<&'v str> {
        self.iter()
            .find(|(k, _)| k.as_bytes() == key)
            .map(|(_, v)| *v)
    }
    #[inline]
    fn get_key_value<'s>(&'s self, key: &'s [u8]) -> Option<(&'k str, &'v str)> {
        self.iter()
            .find(|(k, _)| k.as_bytes() == key)
            .map(|(k, v)| (*k, *v))
    }
}

impl<'k, 'v, const N: usize> ParamsGet<str, [u8; N]> for ParamsStr<'k, 'v> {
    type Key = &'k str;
    type Val = &'v str;
    #[inline]
    fn get<'s>(&'s self, key: &'s [u8; N]) -> Option<&'v str> {
        self.iter()
            .find(|(k, _)| k.as_bytes() == key)
            .map(|(_, v)| *v)
    }
    #[inline]
    fn get_key_value<'s>(&'s self, key: &'s [u8; N]) -> Option<(&'k str, &'v str)> {
        self.iter()
            .find(|(k, _)| k.as_bytes() == key)
            .map(|(k, v)| (*k, *v))
    }
}

impl<'k, 'v> ParamsGet<str, str> for ParamsStr<'k, 'v> {
    type Key = &'k str;
    type Val = &'v str;
    #[inline]
    fn get<'s>(&'s self, key: &'s str) -> Option<&'v str> {
        self.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }
    #[inline]
    fn get_key_value<'s>(&'s self, key: &'s str) -> Option<(&'k str, &'v str)> {
        self.iter().find(|(k, _)| *k == key).map(|(k, v)| (*k, *v))
    }
}

pub trait ParamsGetOwned<Q: ?Sized> {
    type Key<'a>
    where
        Self: 'a;
    type Val<'a>
    where
        Self: 'a;
    fn get<'s>(&'s self, key: &Q) -> Option<Self::Val<'s>>;
    fn get_key_value<'s>(&'s self, key: &Q) -> Option<(Self::Key<'s>, Self::Val<'s>)>;
}

impl ParamsGetOwned<str> for ParamsStrOwned {
    type Key<'s> = &'s str;
    type Val<'s> = &'s str;
    #[inline]
    fn get<'s>(&'s self, key: &str) -> Option<Self::Val<'s>> {
        self.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }
    #[inline]
    fn get_key_value<'s>(&'s self, key: &str) -> Option<(Self::Key<'s>, Self::Val<'s>)> {
        self.iter()
            .find(|(k, _)| k == key)
            .map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

impl ParamsGetOwned<[u8]> for ParamsStrOwned {
    type Key<'s> = &'s str;
    type Val<'s> = &'s str;
    #[inline]
    fn get<'s>(&'s self, key: &[u8]) -> Option<Self::Val<'s>> {
        self.iter()
            .find(|(k, _)| k.as_bytes() == key)
            .map(|(_, v)| v.as_str())
    }
    #[inline]
    fn get_key_value<'s>(&'s self, key: &[u8]) -> Option<(Self::Key<'s>, Self::Val<'s>)> {
        self.iter()
            .find(|(k, _)| k.as_bytes() == key)
            .map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

impl<const N: usize> ParamsGetOwned<[u8; N]> for ParamsStrOwned {
    type Key<'s> = &'s str;
    type Val<'s> = &'s str;
    #[inline]
    fn get<'s>(&'s self, key: &[u8; N]) -> Option<Self::Val<'s>> {
        self.iter()
            .find(|(k, _)| k.as_bytes() == key)
            .map(|(_, v)| v.as_str())
    }
    #[inline]
    fn get_key_value<'s>(&'s self, key: &[u8; N]) -> Option<(Self::Key<'s>, Self::Val<'s>)> {
        self.iter()
            .find(|(k, _)| k.as_bytes() == key)
            .map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

impl ParamsGetOwned<str> for ParamsOwned {
    type Key<'s> = &'s [u8];
    type Val<'s> = &'s [u8];
    #[inline]
    fn get<'s>(&'s self, key: &str) -> Option<Self::Val<'s>> {
        self.iter()
            .find(|(k, _)| k == key.as_bytes())
            .map(|(_, v)| v.as_slice())
    }
    #[inline]
    fn get_key_value<'s>(&'s self, key: &str) -> Option<(Self::Key<'s>, Self::Val<'s>)> {
        self.iter()
            .find(|(k, _)| k == key.as_bytes())
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
    }
}

impl ParamsGetOwned<[u8]> for ParamsOwned {
    type Key<'s> = &'s [u8];
    type Val<'s> = &'s [u8];
    #[inline]
    fn get<'s>(&'s self, key: &[u8]) -> Option<Self::Val<'s>> {
        self.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_slice())
    }
    #[inline]
    fn get_key_value<'s>(&'s self, key: &[u8]) -> Option<(Self::Key<'s>, Self::Val<'s>)> {
        self.iter()
            .find(|(k, _)| k == key)
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
    }
}

impl<const N: usize> ParamsGetOwned<[u8; N]> for ParamsOwned {
    type Key<'s> = &'s [u8];
    type Val<'s> = &'s [u8];
    #[inline]
    fn get<'s>(&'s self, key: &[u8; N]) -> Option<Self::Val<'s>> {
        self.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_slice())
    }
    #[inline]
    fn get_key_value<'s>(&'s self, key: &[u8; N]) -> Option<(Self::Key<'s>, Self::Val<'s>)> {
        self.iter()
            .find(|(k, _)| k == key)
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
    }
}

pub trait ParamsConvertOwned {
    type Owned;
    fn owned(&self) -> Self::Owned;
    #[inline]
    fn into_owned(self) -> Self::Owned
    where
        Self: Sized,
    {
        self.owned()
    }
}

impl<'k, 'v> ParamsConvertOwned for Params<'k, 'v> {
    type Owned = ParamsOwned;
    #[inline]
    fn owned(&self) -> ParamsOwned {
        self.iter().map(|(k, v)| (k.to_vec(), v.to_vec())).collect()
    }
}

impl ParamsConvertOwned for ParamsOwned {
    type Owned = ParamsOwned;
    #[inline]
    fn owned(&self) -> ParamsOwned {
        self.clone()
    }
    #[inline]
    fn into_owned(self) -> ParamsOwned {
        self
    }
}

impl<'k, 'v> ParamsConvertOwned for ParamsStr<'k, 'v> {
    type Owned = ParamsStrOwned;
    #[inline]
    fn owned(&self) -> ParamsStrOwned {
        self.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }
}

impl ParamsConvertOwned for ParamsStrOwned {
    type Owned = ParamsStrOwned;
    #[inline]
    fn owned(&self) -> ParamsStrOwned {
        self.clone()
    }
    #[inline]
    fn into_owned(self) -> ParamsStrOwned {
        self
    }
}
