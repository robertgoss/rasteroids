use bevy::prelude::*;

use super::app_state::AppState;
use super::player::Player;

// Based on bevy example menu code


#[derive(Default)]
pub struct Victory {
    pub player : Option<Player>
}
struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

struct MenuData {
    button_entity: Entity,
    message_entity : Entity
}

fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
    victory : Res<Victory>
) {
    let msg = victory.player.as_ref().map(
        |player| format!("Well done {}!", player.name)
    ).unwrap_or("No winner everyone lost".to_string());
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());
    let button_entity = commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(250.0), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Return to menu",
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
    
    let message_entity = commands.spawn_bundle(TextBundle {
        text: Text::with_section(
            &msg,
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 40.0,
                color: Color::rgb(0.9, 0.9, 0.9),
            },
            TextAlignment {
                horizontal: HorizontalAlign::Center,
                ..Default::default()
            }
        ),
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position: Rect {
                bottom: Val::Px(5.0),
                ..Default::default()
            },
            align_items: AlignItems::Center,
            ..Default::default()
        },
        ..Default::default()
    }).id();
    commands.insert_resource(MenuData { button_entity, message_entity });
}

fn menu(
    mut state: ResMut<State<AppState>>,
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut material) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *material = button_materials.pressed.clone();
                state.set(AppState::MainMenu).unwrap();
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.button_entity).despawn_recursive();
    commands.entity(menu_data.message_entity).despawn_recursive();
}

pub struct VictoryMenuPlugin;

impl Plugin for VictoryMenuPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<ButtonMaterials>()
           .init_resource::<Victory>()
           .add_system_set(
              SystemSet::on_enter(AppState::VictoryMenu)
                .with_system(setup_menu.system())
              )
           .add_system_set(
               SystemSet::on_update(AppState::VictoryMenu)
                 .with_system(menu.system())
                )
           .add_system_set(
               SystemSet::on_exit(AppState::VictoryMenu)
                 .with_system(cleanup_menu.system())
               );
    }
}