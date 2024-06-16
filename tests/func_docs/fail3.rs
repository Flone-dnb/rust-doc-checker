pub const fn just<'a, T, I, E>(seq: T) -> Just<T, I, E>
where
    I: Input<'a>,
    E: ParserExtra<'a, I>,
    I::Token: PartialEq,
    T: OrderedSeq<'a, I::Token> + Clone,
{
    // ...
}
