pub(crate) trait CombineErrors: Iterator {
    type Ok;
    fn try_combine_syn_errors(self) -> syn::Result<Vec<Self::Ok>>;
}

impl<I, T> CombineErrors for I
where
    I: Iterator<Item = syn::Result<T>>,
{
    type Ok = T;

    fn try_combine_syn_errors(self) -> syn::Result<Vec<Self::Ok>> {
        let mut oks: Vec<Self::Ok> = Vec::new();
        let mut errors: Vec<syn::Error> = Vec::new();

        for res in self {
            match res {
                Ok(val) => oks.push(val),
                Err(e) => errors.push(e),
            }
        }

        let mut err_iter = errors.into_iter();
        let mut err = match err_iter.next() {
            // There are no errors
            None => return Ok(oks),
            Some(e) => e,
        };

        for e in err_iter {
            err.combine(e);
        }

        Err(err)
    }
}
