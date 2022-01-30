use simple_test_case::dir_cases;

fn double(n: usize) -> usize {
    n * 2
}

#[dir_cases("tests/test_data")]
#[test]
fn it_works(_path: &str, contents: &str) {
    let (n, expected) = contents.trim().split_once(':').unwrap();
    let n: usize = n.parse().unwrap();
    let expected: usize = expected.parse().unwrap();

    assert_eq!(double(n), expected);
}
