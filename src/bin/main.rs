
use std::thread;
use crossbeam_channel::{bounded,unbounded, Receiver};

#[cfg(target_os = "macos")]
use bevy_math::{DVec2, IVec2, Vec2};
use bevy::window::CompositeAlphaMode;
use bevy::{
    prelude::*,
    window::{Window,PresentMode, WindowPlugin,WindowLevel::AlwaysOnTop,WindowFocused},
    winit::WinitSettings,

};

//use bevy::window::close_on_esc;

extern crate multiinput;
use multiinput::*;


const TRAIL_SIZE:usize = 120; 
const DELTA_DIV:i32 = 4     ; 

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // Setting `transparent` allows the `ClearColor`'s alpha value to take effect
                transparent: true,
                // Disabling window decorations to make it feel more like a widget than a window
                decorations: false,
                window_level:AlwaysOnTop,
                resolution: (1900., 1080.).into(),
                present_mode: PresentMode::AutoVsync,
                position: WindowPosition::At(IVec2{x:0,y:0,}) ,
                #[cfg(target_os = "macos")]
                composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                ..default()
            }),
            ..default()
        }))
        // ClearColor must have 0 alpha, otherwise some color will bleed through
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(MouseTrail{deltas_trail:([(0,0);TRAIL_SIZE]),delta_sum_trail:([(0,0);TRAIL_SIZE]),buffer_head:0,last_moved:0u32})
        .add_event::<StreamEvent>()
        .add_systems(Startup, (
            setup,
            spawn_dots,
            mouse_reader, 
        ))
        .add_systems(Update,( 
            toggle_mouse_passthrough,
            move_dots.after(read_stream_event),
            read_stream,
            read_stream_event,
           
        ))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>,mut windows: Query<&mut Window>) {

    commands.spawn(Camera2dBundle::default());

    commands.spawn((SpriteBundle {
        texture: asset_server.load("MagicMouseMirror.png"),
        transform: Transform::from_xyz(480., -380., 0.).with_scale(Vec3::new(0.20,0.20,0.20)),
        ..default()
        }, 
    ));
    //Toggle passthrough
    let mut window = windows.single_mut();
        window.cursor.hit_test = !window.cursor.hit_test;

}
fn spawn_dots(mut commands: Commands, asset_server: Res<AssetServer>,mut m_trail:ResMut<MouseTrail>) { 

    let dot_core_handle = asset_server.load("Dot_Core.png");
    let dot_glow_handle = asset_server.load("Dot_Glow.png");
    let n_dots = TRAIL_SIZE;
    for i in 0..n_dots { 
        commands.spawn((SpriteBundle {
            texture: dot_glow_handle.clone(),
            sprite: Sprite {

                //color: Color::hsla(0.14,0.84,0.93, 1.0),
                ..default()
            },  
            transform: Transform::from_xyz(480., -380., 1.0),
            ..default()
            }, DotI {index: i as u32, core: false},
        ));
      commands.spawn((SpriteBundle {
            texture: dot_core_handle.clone(),
            sprite: Sprite {
                //color: Color::hsla(0.33,0.20,1.0,1.0),
                ..default()
            },  
            transform: Transform::from_xyz(480.,-380.,2.0),
            ..default()
            }, DotI {index: i as u32,core: true},
        ));
       
    } 

}



fn move_dots(time: Res<Time>, mut m_trail:ResMut<MouseTrail>,  mut events: EventWriter<StreamEvent>, mut dots: Query<(&mut DotI, &mut Transform, &mut Sprite)>) {
  
    //960x540
    //display window bounds     300<x<640, -450<y<-300
    let mut visible = true; 
    if m_trail.last_moved == 0 {
    
        let trail_size:u32 = TRAIL_SIZE as u32; 
        for (mut dot, mut transform, mut sprite) in &mut dots {

            let mut dot_num = dot.index; 
            let mut head  = m_trail.buffer_head as u32; 
            if head == trail_size - 1 { 
                head = 0; 
            }
            else if head < trail_size {
                head = head + 1; 
            }

            
            let mut x_head_transposed =  ( m_trail.delta_sum_trail[head as usize].0/DELTA_DIV) as f32 + 480.0;
            let mut y_head_transposed = -(m_trail.delta_sum_trail[head as usize].1/DELTA_DIV) as f32 - 380.0;

            let mut x_transposed = (m_trail.delta_sum_trail[dot.index as usize].0/DELTA_DIV)  as f32 + 480.0;
            let mut y_transposed = -(m_trail.delta_sum_trail[dot.index as usize].1/DELTA_DIV) as f32 - 380.0; 

            transform.translation.x =  x_transposed;
            transform.translation.y = y_transposed; 
            
            

            
            let mut radial_distance_from_head:f32 = ((x_transposed-x_head_transposed)*(x_transposed-x_head_transposed) + (y_transposed-y_head_transposed)*(y_transposed-y_head_transposed)).sqrt(); 
            let mut distance_from_head = 0; 
            

            if dot_num >= head { 
                distance_from_head = dot_num-head;
            }
            else if dot_num < head { 
                distance_from_head = trail_size - head + dot_num; 
            }


            let mut size_scale:f32 = (trail_size  as f32 -0.75*(distance_from_head as f32))/(trail_size as f32); 
            let mut color_scale:f32 = (1.0-size_scale);
            

            if dot.core == true { 
                transform.translation.z = (2*(trail_size - distance_from_head)) as f32;
                sprite.color = Color::hsla(0.33,0.40,0.045+0.885*size_scale,0.6+0.4*size_scale);
                transform.scale = Vec3::new(1.5*size_scale,1.5*size_scale,1.0); 
            }
            else { 
                transform.translation.z =  (2*(trail_size - distance_from_head)) as f32; 
                sprite.color = Color::hsla(0.14,0.84,0.7+0.2*size_scale, 0.05+0.05*size_scale);
                transform.scale = Vec3::new(1.5*size_scale,1.5*size_scale,1.0); 
            }

        }
        m_trail.last_moved = 0; 
   
    }

    m_trail.last_moved += 1; 
    if m_trail.last_moved > 50 { 
        m_trail.delta_sum_trail = ([(0,0);TRAIL_SIZE]);
        m_trail.buffer_head = 0; 
        m_trail.last_moved = 0; 
    }


  


}
                
fn toggle_mouse_passthrough(keyboard_input: Res<Input<KeyCode>>, mut windows: Query<&mut Window>) {
    if keyboard_input.just_pressed(KeyCode::P) {
        let mut window = windows.single_mut();
        window.cursor.hit_test = !window.cursor.hit_test;
    }
}

#[derive(Component)]
struct DotI {
    index:u32,
    core:bool, 
}

//////////////////Input stream reader. 
#[derive(Resource)]
struct MouseTrail {
    deltas_trail: [(i32,i32);TRAIL_SIZE],
    delta_sum_trail: [(i32,i32);TRAIL_SIZE],
    buffer_head: usize,
    last_moved:u32,
}

#[derive(Resource, Deref)]
struct StreamReceiver(Receiver<(i32,i32)>);

#[derive(Event,Debug)]
struct StreamEvent((i32,i32));

fn mouse_reader(mut commands: Commands) { 

    let mut manager = RawInputManager::new().unwrap();
    manager.register_devices(DeviceType::Joysticks(XInputInclude::True));
    manager.register_devices(DeviceType::Keyboards);
    manager.register_devices(DeviceType::Mice);
    //manager.print_device_list();
    let devices = manager.get_device_list();
    //println!("{:?}", devices);
    
 
    let (tx, rx) = bounded::<(i32,i32)>(10);

    thread::spawn(move || {
        'outer: loop {
            if let Some(event) = manager.get_event() {
                match event {
                    
                    RawEvent::MouseMoveEvent(_,_,_) => {
                        if let RawEvent::MouseMoveEvent(id,dx,dy) = event {
                        //println!("{:?}", event);
                        tx.send((dx,dy)).unwrap();
                        }
                    }
                    _ => (),
                }
                //break 'outer;

            } else {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }); 
    commands.insert_resource(StreamReceiver(rx));
    
}

fn read_stream(receiver: Res<StreamReceiver>, mut events: EventWriter<StreamEvent>) {
    for (per_frame,from_stream) in receiver.try_iter().enumerate() {
        events.send(StreamEvent(from_stream));
    }
}

fn read_stream_event(mut commands: Commands, mut reader: EventReader<StreamEvent>, mut m_trail: ResMut<MouseTrail>) {
   
    for (per_frame, event) in reader.iter().enumerate() {
        let mut head = m_trail.buffer_head;
        if head > 0 { 
            m_trail.buffer_head -= 1; 
            if head < TRAIL_SIZE-1 { 
                let mut delta_sum:(i32,i32) = (m_trail.delta_sum_trail[head+1].0+event.0.0, m_trail.delta_sum_trail[head+1].1+event.0.1);
                delta_sum.0 /= DELTA_DIV; 
                delta_sum.1 /= DELTA_DIV; 

                if delta_sum.0 < 170 && delta_sum.0 > -170 { 
                    m_trail.delta_sum_trail[head].0 = m_trail.delta_sum_trail[head+1].0+event.0.0; 
                }
                else { 
                    m_trail.delta_sum_trail[head].0= -(m_trail.delta_sum_trail[head+1].0-1); 
                }
                if delta_sum.1 < 80 && delta_sum.1 > -80 { 
                    m_trail.delta_sum_trail[head].1 = m_trail.delta_sum_trail[head+1].1+event.0.1;
                }
                else{ 
                    m_trail.delta_sum_trail[head].1 =  -(m_trail.delta_sum_trail[head+1].1-1);
                }
                   
            }
            if head == TRAIL_SIZE-1 { 
                (m_trail.delta_sum_trail[head]) = (m_trail.delta_sum_trail[0].0 + event.0.0,m_trail.delta_sum_trail[0].1+event.0.1);
            }
        }
        else { 
            m_trail.buffer_head = TRAIL_SIZE - 1; 
            m_trail.delta_sum_trail[head] =  (m_trail.delta_sum_trail[head+1].0+event.0.0, m_trail.delta_sum_trail[head+1].1+event.0.1);
        }
        m_trail.last_moved = 0; 
    };
}



