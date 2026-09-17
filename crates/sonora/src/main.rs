#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;
mod logging;

use gpui::{App, AppContext as _, Bounds, Pixels, Size, TitlebarOptions, WindowBounds, WindowOptions, point, px, size};
use router::Destination;
use state::Sonora;
use ui::ActiveTheme as _;
use ui::ThemeKind;
use views::Root;

const LEAST_SIZE: Size<Pixels> = size(px(480.), px(400.));
const FIRST_SIZE: Size<Pixels> = size(px(920.), px(640.));

fn main() {
    logging::init();

    let io = match state::Io::new() {
        Ok(io) => io,
        Err(error) => {
            eprintln!("toolkit: cannot start runtime: {error:#}");
            return;
        }
    };

    let app = gpui_platform::application()
        .with_assets(assets::Assets);

    app.run(move |cx: &mut App| {
        if let Err(error) = assets::Assets.load_fonts(cx) {
            log::error!("toolkit: cannot load bundled fonts: {error:#}");
        }

        state::init(cx, io);
        let start = Destination::Mediatek(router::MediatekTab::Flasher);
        router::init(start, cx);

        let (look, overrides, language, pack, stillness, pace, _remembered) = {
            let settings = Sonora::global(cx).settings.read(cx);
            (
                settings.look(),
                settings.theme_overrides().clone(),
                settings.language().to_owned(),
                settings.icons().to_owned(),
                settings.stillness(),
                settings.pace(),
                settings.system_theme(),
            )
        };
        i18n::set(i18n::resolve(&language));
        icons::set(&pack);
        ui::motion::apply(stillness, pace, cx);
        let reported = match cfg!(any(target_os = "linux", target_os = "freebsd")) {
            true => None,
            false => Some(ThemeKind::reported(cx)),
        };
        ThemeKind::assume(reported.unwrap_or(_remembered));
        ui::Theme::init(look, &overrides, cx);

        open_window(cx);

        cx.activate(true);
    });
}

fn open_window(cx: &mut App) {
    let (placement, display_id) = state::window_placement(LEAST_SIZE, cx)
        .map(|(placement, display_id)| (placement, Some(display_id)))
        .unwrap_or_else(|| {
            (
                WindowBounds::Windowed(Bounds::centered(None, FIRST_SIZE, cx)),
                None,
            )
        });

    let settings = Sonora::global(cx).settings.read(cx);
    let saver = settings.saver();
    let look = settings.look();
    let background = ui::backdrop(look.blur, look.transparent);

    cx.open_window(
        WindowOptions {
            window_bounds: Some(placement),
            display_id,
            window_background: background,
            titlebar: Some(TitlebarOptions {
                title: Some("V1per Servicing Toolkit".into()),
                appears_transparent: true,
                traffic_light_position: Some(point(px(9.), px(9.))),
            }),
            inactive_frame_interval: saver.interval(),
            is_movable: true,
            is_resizable: true,
            app_id: Some("toolkit".into()),
            window_min_size: Some(LEAST_SIZE),
            ..Default::default()
        },
        |window, cx| {
            window.set_rem_size(cx.theme().font_size);
            state::remember_window(window, cx);
            cx.new(|cx| Root::new(window, cx))
        },
    )
    .expect("failed to open window");
}