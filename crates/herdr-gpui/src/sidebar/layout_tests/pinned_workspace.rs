use super::*;

fn pinned(label: &str) -> impl FnOnce(&mut Window, &mut Context<HerdrWindow>) -> HerdrWindow {
    let label = label.to_owned();
    move |window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(Arc::new(snapshot(6)));
        view.live.status = crate::state::ConnectionStatus::Connected;
        view.live.supports_surface = true;
        view.endpoints[0].live = view.live.clone();
        view.config.sidebar_style.pinned_workspace = Some(label);
        view
    }
}

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.simulate_resize(size(px(800.), px(600.)));
    cx.run_until_parked();
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
}

#[gpui::test]
fn a_pinned_workspace_leaves_the_list_for_its_own_row(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(pinned("herdr"));
    draw(cx);
    let row = cx.debug_bounds("pinned-workspace").unwrap();
    let list = cx.debug_bounds("spaces-scroll").unwrap();
    assert!(row.bottom() <= list.top(), "{row:?} above {list:?}");
    assert!(cx.debug_bounds("row-herdr").is_none());
    assert!(cx.debug_bounds("row-another workspace").is_some());
    // Working is not waiting, so the row carries no mark.
    assert!(cx.debug_bounds("pinned-workspace-needs-you").is_none());

    cx.simulate_click(row.center(), Modifiers::default());
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.pending_navigation,
            Some(crate::NavigationTarget::Workspace("w0".into()))
        );
    });

    // Unpinned, the workspace is an ordinary row again.
    view.update(cx, |view, cx| {
        view.config.sidebar_style.pinned_workspace = None;
        cx.notify();
    });
    draw(cx);
    assert!(cx.debug_bounds("pinned-workspace").is_none());
    assert!(cx.debug_bounds("row-herdr").is_some());
}

#[gpui::test]
fn a_blocked_pinned_workspace_says_it_needs_you(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(pinned("herdr"));
    view.update(cx, |view, _| {
        let snapshot = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
        snapshot.workspaces[0].agent_status = AgentStatus::Blocked;
    });
    draw(cx);
    assert!(cx.debug_bounds("pinned-workspace-needs-you").is_some());
}

#[gpui::test]
fn a_pinned_workspace_that_is_not_open_is_created(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(pinned("Butler"));
    draw(cx);
    // The other rows all stay listed.
    assert!(cx.debug_bounds("row-herdr").is_some());
    let row = cx.debug_bounds("pinned-workspace").unwrap();
    cx.simulate_click(row.center(), Modifiers::default());
    cx.run_until_parked();
    // No daemon behind the fixture takes the request, so it reports why
    // instead of navigating anywhere.
    view.read_with(cx, |view, _| {
        assert!(view.pending_navigation.is_none());
        let error = view.local_error.as_deref().unwrap();
        assert!(error.starts_with("Butler: "), "{error}");
    });
}
