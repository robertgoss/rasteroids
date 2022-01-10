use bevy::prelude::*;

use super::app_state::AppState;

// Based on bevy example menu code

struct ButtonMaterials {
    normal: Color,
    hovered: Color,
    pressed: Color,
}

impl FromWorld for ButtonMaterials {
    fn from_world(_: &mut World) -> Self {
        ButtonMaterials {
            normal: Color::rgb(0.15, 0.15, 0.15),
            hovered: Color::rgb(0.25, 0.25, 0.25),
            pressed: Color::rgb(0.35, 0.75, 0.35),
        }
    }
}

struct MenuData {
    button_entity: Entity,
}

fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());
    let button_entity = commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: button_materials.normal.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Play",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .id();
    commands.insert_resource(MenuData { button_entity });
}

fn menu(
    mut state: ResMut<State<AppState>>,
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = button_materials.pressed.into();
                state.set(AppState::InGame).unwrap();
            }
            Interaction::Hovered => {
                *color = button_materials.hovered.into();
            }
            Interaction::None => {
                *color = button_materials.normal.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.button_entity).despawn_recursive();
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonMaterials>()
           .add_system_set(
              SystemSet::on_enter(AppState::MainMenu)
                .with_system(setup_menu.system())
              )
           .add_system_set(
               SystemSet::on_update(AppState::MainMenu)
                 .with_system(menu.system())
                )
           .add_system_set(
               SystemSet::on_exit(AppState::MainMenu)
                 .with_system(cleanup_menu.system())
               );
    }
}