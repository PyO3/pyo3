`experimental-inspect`: annotate `Vec<T>`, `[T; N]`, `SmallVec` and `Cow<[u8]>` inputs with
`_typeshed.SupportsGetItem` and `_typeshed.SupportsLenAndGetItem` instead of `collections.abc.Sequence`,
so that e.g. NumPy arrays are also accepted.
