use simple_test_case::test_case;

#[test_case(1, 2, true; "small and true")]
#[test_case(100, 200, true; "large and true")]
#[test_case(2, 1, false; "small and false")]
#[test_case(200, 100, false; "large and false")]
#[test]
fn simple(a: usize, b: usize, a_less_b: bool) {
    if a_less_b {
        assert!(a < b);
    } else {
        assert!(!(a < b));
    }
}

const FAIL_RESULT_TEST: bool = false;

#[test_case(1, 2, true; "small and true")]
#[test_case(100, 200, true; "large and true")]
#[test_case(2, 1, false; "small and false")]
#[test_case(200, 100, false; "large and false")]
#[test]
fn returning_a_result(a: usize, b: usize, a_less_b: bool) -> Result<(), ()> {
    if a_less_b {
        assert!(a < b);
    } else {
        assert!(!(a < b));
    }

    if FAIL_RESULT_TEST {
        Err(())
    } else {
        Ok(())
    }
}

#[test_case(1, 2, true; "small and true")]
#[test_case(100, 200, true; "large and true")]
#[test_case(2, 1, false; "small and false")]
#[test_case(200, 100, false; "large and false")]
#[tokio::test]
async fn async_simple(a: usize, b: usize, a_less_b: bool) {
    if a_less_b {
        assert!(a < b);
    } else {
        assert!(!(a < b));
    }
}

#[test_case(1, 2, true; "small and true")]
#[test_case(100, 200, true; "large and true")]
#[test_case(2, 1, false; "small and false")]
#[test_case(200, 100, false; "large and false")]
#[tokio::test]
async fn async_returning_a_result(a: usize, b: usize, a_less_b: bool) -> Result<(), ()> {
    if a_less_b {
        assert!(a < b);
    } else {
        assert!(!(a < b));
    }

    if FAIL_RESULT_TEST {
        Err(())
    } else {
        Ok(())
    }
}
