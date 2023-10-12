//! Shows how to display a window in transparent mode.
//!
//! This feature works as expected depending on the platform. Please check the
//! [documentation](https://docs.rs/bevy/latest/bevy/prelude/struct.WindowDescriptor.html#structfield.transparent)
//! for more details.

#[cfg(target_os = "macos")]
use bevy_math::{DVec2, IVec2, Vec2};
use bevy::window::CompositeAlphaMode;
use bevy::{
    prelude::*,
    window::{Window, WindowPlugin,WindowLevel::AlwaysOnTop,PresentMode},
};
use mouse_position::mouse_position::{Mouse};




fn main() {
  

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // Setting `transparent` allows the `ClearColor`'s alpha value to take effect
                present_mode: PresentMode::Immediate,
                transparent: true,
                // Disabling window decorations to make it feel more like a widget than a window
                decorations: false,
                window_level:AlwaysOnTop,
                resolution: (1900., 1080.).into(),
                position: WindowPosition::At(IVec2{x:0,y:0,}) ,
                #[cfg(target_os = "macos")]
                composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                ..default()
            }),
            ..default()
        }))
        // ClearColor must have 0 alpha, otherwise some color will bleed through
        .insert_resource(ClearColor(Color::NONE))
        .add_systems(Startup, setup)
        .init_resource::<PrevMousePos>()
        .add_systems(Update,( 
            sprite_movement,
            toggle_mouse_passthrough,
            print_mouse_events_system,
            sprite_rotation,
        ))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>,mut pmp: ResMut<PrevMousePos>) {
    pmp.x = 0; 
    pmp.y = 0;
    pmp.delta_x=0;
    pmp.delta_y=0;
    commands.spawn(Camera2dBundle::default());
    // commands.spawn((SpriteBundle {
    //     texture: asset_server.load("josh.png"),
    //     transform: Transform::from_xyz(0., 0., 0.),
    //     ..default()
    // },Direction::Up));
    commands.spawn((SpriteBundle {
        texture: asset_server.load("arrow.png"),
        transform: Transform::from_xyz(600., -380., 0.).with_scale(Vec3::new(0.25,0.45,0.25)).with_rotation(Quat::from_rotation_z(0.25*std::f32::consts::PI)),
        ..default()
        }, 
        Rotatable {rotation: Quat::from_rotation_z(0.5*std::f32::consts::PI), pmp: PrevMousePos::default()},
    ));
}


#[derive(Component)]
enum Direction {
    Up,
    Down,
}

#[derive(Component, Default)]
struct Rotatable{
    rotation: Quat,
    pmp:PrevMousePos,
}



fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&mut Direction, &mut Transform)>) {
    for (mut logo, mut transform) in &mut sprite_position {
        match *logo {
            Direction::Up => transform.translation.y += 15. * time.delta_seconds(),
            Direction::Down => transform.translation.y -= 15. * time.delta_seconds(),
        }

        if transform.translation.y > 100. {
            *logo = Direction::Down;
        } else if transform.translation.y < 0. {
            *logo = Direction::Up;
        }
    }
}



fn sprite_rotation(time: Res<Time>, mut sprite_rotation: Query<(&mut Rotatable, &mut Transform)>, pmp: ResMut<PrevMousePos>) {
    for (mut arrow, mut transform) in &mut sprite_rotation {
        let vector = Vec2::from_array([pmp.delta_x as f32, pmp.delta_y as f32]);
        let delta_angle = vector.angle_between(Vec2::Y);
        transform.rotation = Quat::from_rotation_z(delta_angle);
        let mut scale_factor = ((pmp.delta_x*pmp.delta_x + pmp.delta_y*pmp.delta_y)as f32).sqrt()/5f32;
        if scale_factor < 1f32 { scale_factor = 1.0; }
        if scale_factor > 4f32 {scale_factor = 5.0;}

        transform.scale = Vec3::new(0.1*scale_factor,0.2*scale_factor,0.1*scale_factor);
        
        // match *arrow {
        //     Direction::Up => transform.translation.y += 15. * time.delta_seconds(),
        //     Direction::Down => transform.translation.y -= 15. * time.delta_seconds(),
        // }

        // if transform.translation.y > 100. {
        //     *logo = Direction::Down;
        // } else if transform.translation.y < 0. {
        //     *logo = Direction::Up;
        // }
    }
}


fn toggle_mouse_passthrough(keyboard_input: Res<Input<KeyCode>>, mut windows: Query<&mut Window>) {
    if keyboard_input.just_pressed(KeyCode::P) {
        let mut window = windows.single_mut();
        window.cursor.hit_test = !window.cursor.hit_test;
    }
}



#[derive(Resource, Default)]
struct PrevMousePos{
    x:i32,
    y:i32,
    delta_x:i32,
    delta_y:i32,
}

// Prints all mouse events to the console.
/// This system prints out all mouse events as they come in

fn print_mouse_events_system(mut pmp: ResMut<PrevMousePos>){
    let position = Mouse::get_mouse_position();
    match position {
        Mouse::Position { x, y } => {
            if pmp.x != x || pmp.y !=y {
                //pprintln!("x: {}, y: {} pmp x: {} pmp y: {}", x, y,pmp.x,pmp.y);
               // println!("deltax: {}, deltay: {}",pmp.delta_x,pmp.delta_y);
                pmp.delta_x = pmp.x - x;
                pmp.delta_y = pmp.y - y;
                pmp.x=x;
                pmp.y=y;

            }
        },
        Mouse::Error => println!("Error getting mouse position"),
   }
}
