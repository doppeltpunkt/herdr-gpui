use super::*;

#[test]
fn a_pinned_workspace_is_named_by_its_trimmed_label() -> anyhow::Result<()> {
    for config in [Config::default(), Config::parse(DEFAULT_CONFIG)?] {
        assert_eq!(config.sidebar_style.pinned_workspace, None);
    }
    let config = Config::parse("[sidebar]\npinned_workspace = \" Butler \"")?;
    assert_eq!(
        config.sidebar_style.pinned_workspace.as_deref(),
        Some("Butler")
    );
    assert!(config.unknown_keys.is_empty(), "{:?}", config.unknown_keys);
    // A blank label names nothing, and a label is text.
    let blank = Config::parse("[sidebar]\npinned_workspace = \"  \"")?;
    assert_eq!(blank.sidebar_style.pinned_workspace, None);
    assert!(Config::parse("[sidebar]\npinned_workspace = 3").is_err());
    Ok(())
}
