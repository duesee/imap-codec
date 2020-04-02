pub fn join<T: std::fmt::Display>(elements: &[T], sep: &str) -> String {
    elements
        .iter()
        .map(|x| format!("{}", x))
        .collect::<Vec<String>>()
        .join(sep)
}

pub fn join_or_nil<T: std::fmt::Display>(elements: &[T], sep: &str) -> String {
    if elements.is_empty() {
        String::from("nil")
    } else {
        String::from("(")
            + &elements
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join(sep)
            + ")"
    }
}
