use gpui::prelude::*;
use gpui::{AnyView, App, Context, Entity, FocusHandle, Render, StyleRefinement};
use gpui::{Window, div};
use input::WORKSPACE_CONTEXT;
use state::SideTab;
use ui::{Activate, Deselect, Remove, SelectNext, SelectPrevious, shown_listing};

use crate::chrome::{
    Chrome, PlayerBar, SidebarLeft, SidebarRight, TitleBarOptions, ToastStack,
};
use crate::shells::Shell;

pub(crate) struct Workspace {
    sidebar: Entity<SidebarLeft>,
    player_bar: Entity<PlayerBar>,
    sidebar_right: Entity<SidebarRight>,
    toasts: Entity<ToastStack>,
    content: AnyView,
    focus: FocusHandle,
}

impl Workspace {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let sidebar = cx.new(SidebarLeft::new);
        let sidebar_right = cx.new(SidebarRight::new);
        let player_bar = cx.new(|_cx| PlayerBar::new());

        Self {
            sidebar,
            player_bar,
            sidebar_right,
            toasts: cx.new(ToastStack::new),
            content: cx.new(|_cx| div()).into(),
            focus: cx.focus_handle(),
        }
    }

    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        window.focus(&self.focus, cx);
    }

    pub fn toggle_sidebar(&self, cx: &mut Context<Self>) {
        self.sidebar.update(cx, |sidebar, cx| sidebar.toggle(cx));
    }

    pub fn toggle_sidebar_right(&self, cx: &mut Context<Self>) {
        self.sidebar_right.update(cx, |panel, cx| panel.toggle(cx));
    }

    pub fn show_side(&self, tab: SideTab, cx: &mut Context<Self>) {
        self.sidebar_right
            .update(cx, |panel, cx| panel.show(tab, cx));
    }

    pub fn set_content(&mut self, content: AnyView, cx: &mut Context<Self>) {
        self.content = content;
        cx.notify();
    }
}

impl Shell for Workspace {
    fn title_bar(&self, content: Option<AnyView>, cx: &App) -> TitleBarOptions {
        let sidebar = self.sidebar.read(cx);

        TitleBarOptions {
            navigation: true,
            sidebar_open: sidebar.is_open(),
            sidebar_right: Some(self.sidebar_right.read(cx).is_open()),
            offset: sidebar.occupied_width(),
            border: true,
            content,
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let right = self.sidebar_right.read(cx).occupied_width(window);
        self.sidebar
            .update(cx, |sidebar, cx| sidebar.adapt(right, window, cx));
        let left = self.sidebar.read(cx).occupied_width();
        let overlay_width = self.sidebar.read(cx).overlay_width();
        Chrome::publish(left, right, cx);
        let covered = self.sidebar_right.read(cx).covers_content(window);
        let overlay = self.sidebar.read(cx).overlays();

        let sidebar_width = self.sidebar.read(cx).occupied_width();
        let sidebar = match overlay || sidebar_width == gpui::Pixels::ZERO {
            true => self.sidebar.clone().into_any_element(),
            false => self
                .sidebar
                .clone()
                .cached(StyleRefinement::default().w(sidebar_width).h_full())
                .into_any_element(),
        };
        let content = self
            .content
            .clone()
            .cached(StyleRefinement::default().size_full())
            .into_any_element();

        div()
            .relative()
            .flex()
            .flex_col()
            .w_full()
            .flex_1()
            .min_h_0()
            .key_context(WORKSPACE_CONTEXT)
            .track_focus(&self.focus)
            .on_action(|_: &SelectNext, window, cx| {
                if let Some(table) = shown_listing(cx) {
                    table.select_next(window, cx);
                    cx.stop_propagation();
                }
            })
            .on_action(|_: &SelectPrevious, window, cx| {
                if let Some(table) = shown_listing(cx) {
                    table.select_previous(window, cx);
                    cx.stop_propagation();
                }
            })
            .on_action(|_: &Deselect, _, cx| {
                if let Some(table) = shown_listing(cx) {
                    table.deselect(cx);
                    cx.stop_propagation();
                }
            })
            .on_action(|_: &Activate, _, cx| {
                if let Some(table) = shown_listing(cx) {
                    table.activate(cx);
                    cx.stop_propagation();
                }
            })
            .on_action(|_: &Remove, _, cx| {
                if let Some(table) = shown_listing(cx) {
                    table.remove(cx);
                    cx.stop_propagation();
                }
            })
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .when(!overlay, |this| this.child(sidebar))
                    .child(
                        div()
                            .relative()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .min_h_0()
                            .ml(overlay_width)
                            .when(overlay, |this| this.overflow_hidden())
                            .when(covered, |this| this.hidden())
                            .child(content),
                    )
                    .child(self.sidebar_right.clone())
                    .when(overlay, |this| this.child(self.sidebar.clone())),
            )
            .child(
                div()
                    .relative()
                    .child(self.player_bar.clone())
                    .child(self.toasts.clone()),
            )
    }
}