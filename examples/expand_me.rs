use simple_test_case::test_case;

#[test_case(1, 2; "small example")]
#[test_case(100, 200; "large example")]
#[test]
fn example(a: usize, b: usize) {
    assert!(a < b)
}

fn main() {}
