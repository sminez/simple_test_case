use simple_test_case::test_case;

fn double(n: usize) -> usize {
    n * 2
}

#[test_case(1, 2; "small")]
#[test_case(100, 200; "large")]
#[test]
fn simple(a: usize, b: usize) {
    assert_eq!(double(a), b)
}

// #[test_case(200, 100; "failing")]  // XXX: uncomment to verify failures from results are working
#[test_case(1, 2; "simple passing")]
#[test_case(100, 200; "other simple passing")]
#[test]
fn returning_a_result(a: usize, b: usize) -> Result<(), (usize, usize)> {
    if double(a) != b {
        Err((double(a), b))
    } else {
        Ok(())
    }
}

#[test_case(1, 2; "small")]
#[test_case(100, 200; "large")]
#[tokio::test]
async fn async_simple(a: usize, b: usize) {
    assert_eq!(double(a), b)
}

// #[test_case(200, 100; "failing")]  // XXX: uncomment to verify failures from results are working
#[test_case(1, 2; "simple passing")]
#[test_case(100, 200; "other simple passing")]
#[tokio::test]
async fn async_returning_a_result(a: usize, b: usize) -> Result<(), (usize, usize)> {
    if double(a) != b {
        Err((double(a), b))
    } else {
        Ok(())
    }
}

#[test_case(1, 2; "small")]
#[test_case(100, 200; "large")]
#[test]
#[should_panic(expected = "should panic here")]
fn should_panic(a: usize, b: usize) {
    assert_eq!(double(a), b);
    panic!("should panic here");
}

#[test_case(1, 2; "small")]
#[test_case(100, 200; "large")]
#[tokio::test]
#[should_panic(expected = "should panic here")]
async fn async_should_panic(a: usize, b: usize) {
    assert_eq!(double(a), b);
    panic!("should panic here");
}
