/// Some docs.
pub struct FunctionInfo<'src> {
    /// Some docs.
    pub field: MyGeneric<&'src str, Foo, Bar>,

    some_private: usize,
}
