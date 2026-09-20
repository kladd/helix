use super::*;

#[tokio::test(flavor = "multi_thread")]
async fn substitute_first_match_on_each_line() -> anyhow::Result<()> {
    test((
        indoc! {"\
            #[x foo foo
            x foo foo
            x|]#
            "},
        ":s/foo/ba/<ret>",
        indoc! {"\
            #[x ba foo
            x ba foo
            x|]#
            "},
    ))
    .await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn substitute_global_every_match_on_each_line() -> anyhow::Result<()> {
    test((
        indoc! {"\
            #[x foo foo
            x foo foo
            x|]#
            "},
        ":s/foo/ba/g<ret>",
        indoc! {"\
            #[x ba ba
            x ba ba
            x|]#
            "},
    ))
    .await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn substitute_first_match_on_each_line_with_captures() -> anyhow::Result<()> {
    test((
        indoc! {"\
            #[x foo foo
            x foo foo
            x|]#
            "},
        ":s/f(o+)/$1/<ret>",
        indoc! {"\
            #[x oo foo
            x oo foo
            x|]#
            "},
    ))
    .await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn substitute_all_first_match_on_each_line() -> anyhow::Result<()> {
    test((
        indoc! {"\
            foo foo
            foo foo
            #[x|]#
            "},
        ":%s/foo/ba/<ret>",
        indoc! {"\
            ba foo
            ba foo
            #[x|]#
            "},
    ))
    .await?;

    Ok(())
}
