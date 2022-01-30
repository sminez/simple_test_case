use simple_test_case::dir_cases;

fn double(n: usize) -> usize {
    n * 2
}

fn parse_test_file(contents: &str) -> anyhow::Result<(usize, usize)> {
    let (n, expected) = contents.trim().split_once(':').expect("invalid test file");

    Ok((n.parse()?, expected.parse()?))
}

#[dir_cases("tests/test_data")]
#[test]
fn it_works(_path: &str, contents: &str) -> anyhow::Result<()> {
    let (n, expected) = parse_test_file(contents)?;

    assert_eq!(double(n), expected);
    Ok(())
}

#[dir_cases("tests/test_data")]
#[tokio::test]
async fn it_async_works(_path: &str, contents: &str) -> anyhow::Result<()> {
    let (n, expected) = parse_test_file(contents)?;

    assert_eq!(double(n), expected);
    Ok(())
}
