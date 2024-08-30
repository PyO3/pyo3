// Copied from the standard library: https://doc.rust-lang.org/stable/std/iter/trait.TrustedLen.html.

pub(super) unsafe trait TrustedLen: Iterator {}

unsafe impl TrustedLen for std::char::ToLowercase {}
unsafe impl TrustedLen for std::char::ToUppercase {}
unsafe impl TrustedLen for std::str::Bytes<'_> {}
unsafe impl<T> TrustedLen for std::slice::Chunks<'_, T> {}
unsafe impl<T> TrustedLen for std::slice::ChunksMut<'_, T> {}
unsafe impl<T> TrustedLen for std::slice::ChunksExact<'_, T> {}
unsafe impl<T> TrustedLen for std::slice::ChunksExactMut<'_, T> {}
unsafe impl<T> TrustedLen for std::slice::RChunks<'_, T> {}
unsafe impl<T> TrustedLen for std::slice::RChunksMut<'_, T> {}
unsafe impl<T> TrustedLen for std::slice::RChunksExact<'_, T> {}
unsafe impl<T> TrustedLen for std::slice::RChunksExactMut<'_, T> {}
unsafe impl<T> TrustedLen for std::slice::Windows<'_, T> {}
unsafe impl<T> TrustedLen for std::iter::Empty<T> {}
unsafe impl<T> TrustedLen for std::iter::Once<T> {}
unsafe impl<T> TrustedLen for std::option::IntoIter<T> {}
unsafe impl<T> TrustedLen for std::option::Iter<'_, T> {}
unsafe impl<T> TrustedLen for std::option::IterMut<'_, T> {}
unsafe impl<T> TrustedLen for std::result::IntoIter<T> {}
unsafe impl<T> TrustedLen for std::result::Iter<'_, T> {}
unsafe impl<T> TrustedLen for std::result::IterMut<'_, T> {}
unsafe impl<T> TrustedLen for std::collections::vec_deque::IntoIter<T> {}
unsafe impl<T> TrustedLen for std::collections::vec_deque::Iter<'_, T> {}
unsafe impl<T> TrustedLen for std::slice::Iter<'_, T> {}
unsafe impl<T> TrustedLen for std::slice::IterMut<'_, T> {}
unsafe impl<T> TrustedLen for std::vec::Drain<'_, T> {}
unsafe impl<T> TrustedLen for std::vec::IntoIter<T> {}
unsafe impl<T, const N: usize> TrustedLen for std::array::IntoIter<T, N> {}
unsafe impl<T> TrustedLen for std::ops::Range<T>
where
    T: TrustedStep,
    std::ops::Range<T>: Iterator,
{
}
unsafe impl<T> TrustedLen for std::ops::RangeFrom<T>
where
    T: TrustedStep,
    std::ops::RangeFrom<T>: Iterator,
{
}
unsafe impl<T> TrustedLen for std::ops::RangeInclusive<T>
where
    T: TrustedStep,
    std::ops::RangeInclusive<T>: Iterator,
{
}

unsafe impl<'a, I, T> TrustedLen for std::iter::Cloned<I>
where
    T: Clone + 'a,
    I: TrustedLen<Item = &'a T>,
{
}
unsafe impl<'a, I, T> TrustedLen for std::iter::Copied<I>
where
    T: Copy + 'a,
    I: TrustedLen<Item = &'a T>,
{
}
unsafe impl<T> TrustedLen for std::iter::Repeat<T> where T: Clone {}
unsafe impl<A, B> TrustedLen for std::iter::Chain<A, B>
where
    A: TrustedLen,
    B: TrustedLen<Item = A::Item>,
{
}
unsafe impl<A, B> TrustedLen for std::iter::Zip<A, B>
where
    A: TrustedLen,
    B: TrustedLen,
{
}
unsafe impl<A, F> TrustedLen for std::iter::OnceWith<F> where F: FnOnce() -> A {}
unsafe impl<A, F> TrustedLen for std::iter::RepeatWith<F> where F: FnMut() -> A {}
unsafe impl<B, I, F> TrustedLen for std::iter::Map<I, F>
where
    I: TrustedLen,
    F: FnMut(I::Item) -> B,
{
}
unsafe impl<I> TrustedLen for std::iter::Enumerate<I> where I: TrustedLen {}
unsafe impl<I> TrustedLen for std::iter::Fuse<I> where I: TrustedLen {}
unsafe impl<I> TrustedLen for std::iter::Peekable<I> where I: TrustedLen {}
unsafe impl<I> TrustedLen for std::iter::Rev<I> where I: TrustedLen + DoubleEndedIterator {}
unsafe impl<I> TrustedLen for std::iter::Take<I> where I: TrustedLen {}

unsafe impl<I> TrustedLen for &mut I where I: TrustedLen + ?Sized {}

unsafe trait TrustedStep {}

unsafe impl TrustedStep for char {}
unsafe impl TrustedStep for i8 {}
unsafe impl TrustedStep for i16 {}
unsafe impl TrustedStep for i32 {}
unsafe impl TrustedStep for i64 {}
unsafe impl TrustedStep for i128 {}
unsafe impl TrustedStep for isize {}
unsafe impl TrustedStep for u8 {}
unsafe impl TrustedStep for u16 {}
unsafe impl TrustedStep for u32 {}
unsafe impl TrustedStep for u64 {}
unsafe impl TrustedStep for u128 {}
unsafe impl TrustedStep for usize {}
unsafe impl TrustedStep for std::net::Ipv4Addr {}
unsafe impl TrustedStep for std::net::Ipv6Addr {}
