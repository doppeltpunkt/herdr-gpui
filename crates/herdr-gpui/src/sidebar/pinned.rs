//! The workspace `[sidebar] pinned_workspace` names: a row of its own at the
//! sidebar's top, above the spaces list, which then leaves that workspace
//! out. Only this computer's Herdr is searched, by label, so the row follows
//! the workspace when the daemon gives it a new ID. The previous, next, and
//! numbered shortcuts still count it where the daemon lists it.

use super::{
    agents::{Indicators, status_indicator},
    cell::RowState,
    label_text,
    layout::SidebarLook,
    line_height,
    metrics::glyph_width,
};
use crate::{HerdrWindow, NavigationTarget, endpoint::LOCAL};
use gpui::{prelude::*, *};
use herdr_client::{
    Method,
    protocol::{AgentStatus, ClientShellWorkspace},
};

const NEEDS_YOU: &str = "needs you";

/// Whether `workspace` is the one `label` pins: a top-level row of that name.
/// A linked worktree sits under its checkout, so it is never pulled out.
fn pins(workspace: &ClientShellWorkspace, label: &str) -> bool {
    workspace.label == label && !crate::worktree_identity::is_linked(workspace)
}

impl HerdrWindow {
    /// The pinned workspace, while this computer's Herdr has one open.
    fn pinned_workspace(&self) -> Option<&ClientShellWorkspace> {
        let label = self.config.sidebar_style.pinned_workspace.as_deref()?;
        let live = if self.selected_endpoint == 0 {
            &self.live
        } else {
            &self.endpoints.first()?.live
        };
        live.snapshot
            .as_ref()?
            .workspaces
            .iter()
            .find(|workspace| pins(workspace, label))
    }

    /// Drops the pinned workspace from endpoint `index`'s spaces list, which
    /// `entries` holds as `(workspace index, child, group)`. A group's head
    /// stays, so its worktrees keep the row they hang from.
    pub(super) fn unlist_pinned(
        &self,
        index: usize,
        workspaces: &[ClientShellWorkspace],
        entries: &mut Vec<(usize, bool, Option<String>)>,
    ) {
        let Some(pinned) = self.pinned_workspace().filter(|_| index == 0) else {
            return;
        };
        entries.retain(|(row, child, group)| {
            *child || group.is_some() || workspaces[*row].workspace_id != pinned.workspace_id
        });
    }

    /// The pinned row, when a workspace is pinned: its name, its status, and
    /// a mark while it waits on the user. Without the workspace open the row
    /// is muted and creates it.
    pub(super) fn pinned_row(
        &self,
        look: SidebarLook,
        width: f32,
        indicators: Indicators,
        cx: &mut Context<Self>,
    ) -> Option<Stateful<Div>> {
        let label = self.config.sidebar_style.pinned_workspace.clone()?;
        let font = &self.config.sidebar;
        let theme = &self.theme;
        let layout = look.density;
        let gap = layout.host_gap();
        let workspace = self.pinned_workspace();
        let status = workspace.map(|workspace| workspace.agent_status);
        let selected = self.selected_endpoint == 0 && workspace.is_some_and(|w| w.focused);
        let waiting = status == Some(AgentStatus::Blocked);
        let icon = (font.size * 0.9).round();
        let mark_size = (font.size * 0.8).round();
        let mark_width = if waiting {
            NEEDS_YOU.len() as f32 * glyph_width(font) * 0.8 + 8. + gap
        } else {
            0.
        };
        let label_width =
            (look.content_width(width) - icon - gap - indicators.width(font) - gap - mark_width)
                .max(0.);
        let foreground = if workspace.is_some() {
            theme.foreground
        } else {
            theme.muted
        };
        let accent = indicators.color(AgentStatus::Blocked);
        let description = match status {
            Some(status) => format!("{label}, {}", super::status_text(status)),
            None => format!("{label}, not open: click to create it"),
        };
        Some(
            div()
                .id("pinned-workspace")
                .debug_selector(|| "pinned-workspace".into())
                .role(Role::Button)
                .aria_label(description)
                .aria_selected(selected)
                .flex_none()
                .relative()
                .h(px(line_height(font)
                    + 2. * layout.host_padding()
                    + look.chrome_height()))
                .px(px(look.content_x()))
                .flex()
                .items_center()
                .gap(px(gap))
                .border_b_1()
                .border_color(rgb(theme.active))
                .child(look.highlight(
                    "pinned-workspace",
                    RowState {
                        selected,
                        ..RowState::default()
                    },
                    theme,
                ))
                .text_color(rgb(foreground))
                .cursor_pointer()
                .child(
                    svg()
                        .path("icons/bell.svg")
                        .size(px(icon))
                        .flex_none()
                        .text_color(rgb(if waiting { accent } else { foreground })),
                )
                .child(
                    div()
                        .debug_selector(|| "pinned-workspace-label".into())
                        // A fixed width, as on the other rows: text measured
                        // from a zero flex basis would not hold its room.
                        .w(px(label_width))
                        .flex_none()
                        .overflow_hidden()
                        .child(
                            div()
                                .w(px(label_width))
                                .truncate()
                                .child(label_text(&label)),
                        ),
                )
                .when(waiting, |row| {
                    row.child(
                        div()
                            .debug_selector(|| "pinned-workspace-needs-you".into())
                            .flex_none()
                            .px(px(4.))
                            .rounded_full()
                            .bg(rgb(accent))
                            .text_color(rgb(theme.background))
                            .text_size(px(mark_size))
                            .child(NEEDS_YOU),
                    )
                })
                .children(status.map(|status| status_indicator(status, font, indicators)))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.open_pinned(cx);
                    window.focus(&this.focus, cx);
                })),
        )
    }

    /// Focuses the pinned workspace, or asks this computer's Herdr for a new
    /// one with its label. The new one is empty: whatever the user runs
    /// there is theirs to start.
    // ponytail: a second click before the daemon's snapshot names the new
    // workspace creates another one; a pending-create flag would fence it.
    fn open_pinned(&mut self, cx: &mut Context<Self>) {
        if self.menu.page.is_some() {
            return;
        }
        let Some(label) = self.config.sidebar_style.pinned_workspace.clone() else {
            return;
        };
        if let Some(id) = self.pinned_workspace().map(|w| w.workspace_id.clone()) {
            self.navigate_endpoint(LOCAL, NavigationTarget::Workspace(&id), cx);
            return;
        }
        if !self.select_endpoint(LOCAL, cx) {
            self.local_error = Some(format!("{label}: this computer's Herdr is not available"));
            cx.notify();
            return;
        }
        let params = serde_json::json!({"label": label, "focus": true});
        if !self.request_focus_change("Open workspace", None, |handle, boot| {
            handle.request(boot, Method::WorkspaceCreate, params)
        }) && self.local_error.is_none()
        {
            self.local_error = Some(format!("{label}: not connected to this computer's Herdr"));
        }
        cx.notify();
    }
}
