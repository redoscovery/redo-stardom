use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::{EguiContexts, EguiPlugin};

mod data_loading;
mod game_bridge;
mod states;
mod ui;

use states::AppState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "REDÓ Stardom".to_string(),
                resolution: WindowResolution::new(800, 600),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .init_state::<AppState>()
        .add_plugins(ui::UiPlugin)
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, configure_egui_fonts)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn configure_egui_fonts(mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let font_data = include_bytes!("../../../assets/fonts/NotoSansCJKtc-Regular.otf");

    let mut fonts = bevy_egui::egui::FontDefinitions::default();
    fonts.font_data.insert(
        "NotoSansCJKtc".to_string(),
        bevy_egui::egui::FontData::from_static(font_data).into(),
    );

    // Prepend to Proportional and Monospace so CJK glyphs are found first
    fonts
        .families
        .entry(bevy_egui::egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "NotoSansCJKtc".to_string());
    fonts
        .families
        .entry(bevy_egui::egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "NotoSansCJKtc".to_string());

    ctx.set_fonts(fonts);
}
