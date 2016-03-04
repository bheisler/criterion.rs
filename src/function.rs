pub struct One<T> {
    pub function: ::Function<T>,
}

pub struct Many<T> {
    // TODO Can we replace `Vec` with an `IntoIterator` implementor?
    pub functions: Vec<::Function<T>>,
}
