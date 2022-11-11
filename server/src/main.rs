use log::{trace, info, warn};
use renet::{
    RenetConnectionConfig, RenetServer, ServerAuthentication,
    ServerConfig, ServerEvent, NETCODE_USER_DATA_BYTES,
};
use store::{Player, BaseStats, EndGameReason, GameEvent};
use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, SystemTime},
	thread,
};

const PROTOCOL_ID: u64 = 7;
const MAX_CLIENTS: usize = 64;

fn main() {
    use store::GameEvent::*;
    env_logger::init();

    let server_addr: SocketAddr =
		"127.0.0.1:5000".parse().unwrap();
		
        // format!("{}:{}", env!("HOST"), env!("PORT")).parse().unwrap();

    let mut server: RenetServer = RenetServer::new(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        ServerConfig::new(
            MAX_CLIENTS,
            PROTOCOL_ID,
            server_addr,
            ServerAuthentication::Unsecure,
        ),
        RenetConnectionConfig::default(),
        UdpSocket::bind(server_addr).unwrap(),
    )
    .unwrap();

    trace!("Server listening on {}", server_addr);

    let mut game_state = store::GameState::default();
    let delta_time = Duration::from_millis(16);
    let channel_id = 0;

    loop {
        // Receive new messages and update clients
        server.update(delta_time).unwrap();

        // Check for client connections/disconnections
        while let Some(event) = server.get_event() {
            match event {
                ServerEvent::ClientConnected(id, user_data) => {
					for (player_id, player) in
					game_state.players.iter()
                    {
						let event = PlayerJoined {
							player_id: *player_id,
                            player: player.clone(),
                        };
						server.send_message(id, channel_id, bincode::serialize(&event).unwrap());
                    }
					
					// Add the new player to the game
					let event = PlayerJoined {
						player_id: id,
						player: name_from_user_data(&user_data)
					};
					game_state.trigger(&event);
					
					// Tell all players that a new player has joined
                    server.broadcast_message(channel_id, bincode::serialize(&event).unwrap());
					
					info!("Client {} connected", id);
					if game_state.players.len() == 2 {
						let event = BeginGame { goes_first: id };
						game_state.trigger(&event);

						server.broadcast_message(channel_id, bincode::serialize(&event).unwrap());
						trace!("The game has begun");
					}
                }
                ServerEvent::ClientDisconnected(id) => {
					// First trigger a disconnect event
					let event = PlayerDisconnected { player_id: id };
					game_state.trigger(&event);
					server.broadcast_message(0, bincode::serialize(&event).unwrap());
					info!("Client {} disconnected", id);

					let event = EndGame {
						reason: EndGameReason::PlayerSurrender { player_id: id },
					};
					game_state.trigger(&event);
					server.broadcast_message(0, bincode::serialize(&event).unwrap());
                }
            }
        }

        // Receive message from channel
        for client_id in server.clients_id().into_iter() {
			while let Some(message) = server.receive_message(client_id, 0) {
                if let Ok(event) = bincode::deserialize::<GameEvent>(&message) {
                    if game_state.validate(&event) {
                        game_state.trigger(&event);
                        trace!("Player {} sent:\n\t{:#?}", client_id, event);
                        server.broadcast_message(0, bincode::serialize(&event).unwrap());

                        // Determine if a player has won the game
                        if let Some(winner) = game_state.determine_winner() {
                            let event = EndGame {
                                reason: EndGameReason::PlayerWon { winner },
                            };
                            server.broadcast_message(0, bincode::serialize(&event).unwrap());
                        }
                    } else {
                        warn!("Player {} sent invalid event:\n\t{:#?}", client_id, event);
                    }
                }
            }
        }

		server.send_packets().unwrap();
        thread::sleep(Duration::from_millis(50));
    }
}

/// Utility function for extracting a players name from renet user data
fn name_from_user_data(user_data: &[u8; NETCODE_USER_DATA_BYTES]) -> Player {
	trace!("USER_DATA {:?}", user_data);
    // let mut buffer = [0u8; 8];
    // buffer.copy_from_slice(&user_data[0..8]);
    // let mut len = u64::from_le_bytes(buffer) as usize;
    // len = len.min(NETCODE_USER_DATA_BYTES - 8);
    // let data = user_data[8..len + 8].to_vec();
    // String::from_utf8(data).unwrap()
	Player { name: "aaa".to_string(), base_stats: BaseStats { health: 1000, attack: 100, defense: 100  }, player_type: store::PlayerType::Bot }
}