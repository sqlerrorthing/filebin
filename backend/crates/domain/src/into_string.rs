pub trait IntoOptionalString {
    fn as_ref(&self) -> Option<&str>;
}

impl IntoOptionalString for &str {
    fn as_ref(&self) -> Option<&str> {
        Some(self)
    }
}

impl IntoOptionalString for String {
    fn as_ref(&self) -> Option<&str> {
        Some(self)
    }
}

impl<S> IntoOptionalString for Option<S>
where
    S: IntoOptionalString
{
    fn as_ref(&self) -> Option<&str> {
        self.as_ref().and_then(|s| s.as_ref())
    }
}

impl<S> IntoOptionalString for &S
where 
    S: IntoOptionalString
{
    fn as_ref(&self) -> Option<&str> {
        S::as_ref(self)
    }
}