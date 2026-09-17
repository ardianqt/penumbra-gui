use gpui::prelude::*;
use gpui::{Context, Render, Window, div, px};
use state::{AppSettings, Sonora};
use ui::{ActiveTheme as _, Button, Switch};

pub(crate) struct SettingsView {
    settings: Entity<AppSettings>,
}

impl SettingsView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let settings = Sonora::global(cx).settings.clone();
        cx.observe(&settings, |_, _, cx| cx.notify()).detach();
        Self { settings }
    }

    pub fn select(&mut self, _tab: router::SettingsTab, _cx: &mut Context<Self>) {
        cx.notify();
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.theme();
        let settings = self.settings.read(cx);

        div()
            .flex()
            .flex_col()
            .size_full()
            .p_8()
            .overflow_y_scroll()
            .child(
                div()
                    .text_2xl()
                    .child("Settings"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .mt_6()
                    .gap_4()
                    .child(
                        div()
                            .child(
                                div()
                                    .text_lg()
                                    .child("Appearance"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .mt_2()
                                    .child(div().child("Theme"))
                                    .child(
                                        Button::new("theme-toggle")
                                            .ghost()
                                            .label(settings.theme().to_string())
                                            .on_click(cx.listener(|_, _, _, _| {})),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .child(
                                div()
                                    .text_lg()
                                    .child("About"),
                            )
                            .child(
                                div()
                                    .mt_2()
                                    .text_color(theme.muted_foreground)
                                    .child("V1per Servicing Toolkit v1.0"),
                            ),
                    ),
            )
    }
}