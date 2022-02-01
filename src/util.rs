pub(crate) fn slugify_path(p: &str) -> String {
    let mut s: String = p
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();

    if s.starts_with(|c: char| c.is_numeric()) {
        s.insert(0, '_');
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_delimited() {
        assert_eq!(&slugify_path("simple example"), "simple_example");
    }

    #[test]
    fn leading_num() {
        assert_eq!(&slugify_path("99 bottles of beer"), "_99_bottles_of_beer");
    }

    #[test]
    fn punctuation() {
        assert_eq!(&slugify_path("some-file_path.txt"), "some_file_path_txt");
    }
}
