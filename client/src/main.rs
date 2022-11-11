#![allow(unused)]
use bevy::{prelude::*, window::close_on_esc};
use bevy_renet::{run_if_client_connected, RenetClientPlugin};
use renet::{
    ClientAuthentication, RenetClient, RenetConnectionConfig, RenetError, NETCODE_USER_DATA_BYTES,
};
use std::{net::UdpSocket, time::SystemTime};
use store::{EndGameReason, GameEvent, GameState};

const PROTOCOL_ID: u64 = 7;

fn main() {
	// Get username from stdin args
	let args = std::env::args().collect::<Vec<String>>();
	let username = &args[1];

	App::new()
		.insert_resource(WindowDescriptor {
			title: format!("Thruga <{}>", username),
			width: 1024.0,
			height: 720.0,
			..default()
		})
		.insert_resource(ClearColor(Color::Rgba { red: 0.5, green: 0.5, blue: 0.5, alpha: 0.5 }))
		.add_plugins(DefaultPlugins)
        // Renet setup
        // .add_plugin(RenetClientPlugin)
        // .insert_resource(new_renet_client(&username).unwrap())
        // .add_system(handle_renet_error)
        // .add_system_to_stage(
        //     CoreStage::PostUpdate,
        //     receive_events_from_server.with_run_criteria(run_if_client_connected),
        // )
		// Add our game state and register GameEvent as a bevy event
		.insert_resource(GameState::default())
		.add_event::<GameEvent>()
		// Add setup function to spawn UI and board graphics
		.add_startup_system(setup)
		.add_system(close_on_esc)
		.run();
}

////////// RENET NETWORKING //////////
fn new_renet_client(username: &String) -> anyhow::Result<RenetClient> {
    // let server_addr = format!("{}:{}", env!("HOST"), env!("PORT")).parse()?;
    let server_addr = "127.0.0.1:5000".parse()?;
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let client_id = current_time.as_millis() as u64;

    // Place username in user data
    let mut user_data = [0u8; NETCODE_USER_DATA_BYTES];
    if username.len() > NETCODE_USER_DATA_BYTES - 8 {
        panic!("Username is too big");
    }
    user_data[0..8].copy_from_slice(&(username.len() as u64).to_le_bytes());
    user_data[8..username.len() + 8].copy_from_slice(username.as_bytes());

    let client = RenetClient::new(
        current_time,
        socket,
        client_id,
        RenetConnectionConfig::default(),
        ClientAuthentication::Unsecure {
            client_id,
            protocol_id: crate::PROTOCOL_ID,
            server_addr,
            user_data: Some(user_data),
        },
    )?;

    Ok(client)
}

// If there's any error network we just panic 🤷‍♂️
fn handle_renet_error(mut renet_error: EventReader<RenetError>) {
    for err in renet_error.iter() {
        panic!("{}", err);
    }
}

fn receive_events_from_server(
    mut client: ResMut<RenetClient>,
    mut game_state: ResMut<GameState>,
    mut game_events: EventWriter<GameEvent>,
) {
    while let Some(message) = client.receive_message(0) {
        // Whenever the server sends a message we know that it must be a game event
        let event: GameEvent = bincode::deserialize(&message).unwrap();
        trace!("{:#?}", event);

        // We trust the server - It's always been good to us!
        // No need to validate the events it is sending us
        game_state.trigger(&event);

        // Send the event into the bevy event system so systems can react to it
        game_events.send(event);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.spawn_bundle(Camera2dBundle::default());

	// Spawn board background
	commands.spawn_bundle(SpriteBundle {
		// transform: Transform::from_xyz(0.0, -30.0, 0.0),
		sprite: Sprite {
			custom_size: Some(Vec2::new(1024.0, 720.0)),
			..default()
		},
		texture: asset_server.load("bg_battle.png").into(),
		..default()
	});
}