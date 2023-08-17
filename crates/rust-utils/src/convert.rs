pub trait FromExt: Sized {
    fn from_<T>(t: T) -> Self
    where
        Self: From<T>,
    {
        From::from(t)
    }
}

pub trait IntoExt: Sized {
    fn into_<T>(self) -> T
    where
        Self: Into<T>,
    {
        Into::into(self)
    }
}

pub trait TryFromExt: Sized {
    fn try_from_<T>(t: T) -> Result<Self, <Self as TryFrom<T>>::Error>
    where
        Self: TryFrom<T>,
    {
        TryFrom::try_from(t)
    }
}

pub trait TryIntoExt: Sized {
    fn try_into_<T>(self) -> Result<T, <Self as TryInto<T>>::Error>
    where
        Self: TryInto<T>,
    {
        TryInto::try_into(self)
    }
}

impl<T> FromExt for T {}
impl<T> IntoExt for T {}

impl<T> TryFromExt for T {}
impl<T> TryIntoExt for T {}

pub fn from<T, U>(t: T) -> U
where
    U: From<T>,
{
    U::from(t)
}

pub fn into<T, U>(t: T) -> U
where
    T: Into<U>,
{
    T::into(t)
}

pub fn try_from<T, U>(t: T) -> Result<U, <U as TryFrom<T>>::Error>
where
    U: TryFrom<T>,
{
    U::try_from(t)
}

pub fn try_into<T, U>(t: T) -> Result<U, <T as TryInto<U>>::Error>
where
    T: TryInto<U>,
{
    T::try_into(t)
}
