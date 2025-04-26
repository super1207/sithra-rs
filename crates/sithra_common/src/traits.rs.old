pub trait CollectError {
    type Error;
    type Item;
    fn collect_error(self, errors: &mut Vec<Self::Error>) -> impl Iterator<Item = Self::Item>;
}

impl<I, E, T> CollectError for I
where
    I: IntoIterator<Item = Result<T, E>>,
{
    type Error = E;
    type Item = T;
    fn collect_error(self, errors: &mut Vec<Self::Error>) -> impl Iterator<Item = Self::Item> {
        self.into_iter().filter_map(|x| match x {
            Ok(v) => Some(v),
            Err(e) => {
                errors.push(e);
                None
            }
        })
    }
}
