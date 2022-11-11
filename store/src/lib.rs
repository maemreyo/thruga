use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub base_stats: BaseStats,
    pub player_type: PlayerType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseStats {
    pub health: i64,
    pub attack: i64,
    pub defense: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlayerType {
    Bot,
    Human,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize
)]
pub enum Stage {
    PreGame,
    InGame,
    Ended,
}

type PlayerId = u64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    pub stage: Stage,
    pub board: [PlayerType; 2],
    pub active_player_id: PlayerId,
    pub players: HashMap<PlayerId, Player>,
    pub history: Vec<GameEvent>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            stage: Stage::PreGame,
            board: [PlayerType::Bot, PlayerType::Bot],
            active_player_id: 0,
            players: HashMap::new(),
            history: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Deserialize)]
pub enum EndGameReason {
    PlayerSurrender { player_id: PlayerId },
    PlayerWon { winner: PlayerId },
}

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub enum GameEvent {
    BeginGame { goes_first: PlayerId },
    EndGame { reason: EndGameReason },
    PlayerJoined { player_id: PlayerId, player: Player },
    PlayerDisconnected { player_id: PlayerId },
    PlayerAttack { player_id: PlayerId, enemy_id: PlayerId },
}

impl GameState {
    pub fn validate(&self, event: &GameEvent) -> bool {
        use GameEvent::*;
        match event {
            BeginGame { goes_first } => {
                let player_is_unknown =
                    self.players.contains_key(goes_first);
                if self.stage != Stage::PreGame || player_is_unknown {
                    return false;
                }
            }
            EndGame { reason } => match reason {
                EndGameReason::PlayerWon { winner: _ } => {
                    if self.stage != Stage::InGame {
                        return false;
                    }
                }
                EndGameReason::PlayerSurrender { player_id: _ } => {
                    if self.stage != Stage::InGame {
                        return false;
                    }
                }
            },
            PlayerJoined { player_id, player: _ } => {
                if self.players.contains_key(player_id) {
                    return false;
                }
            }
            PlayerDisconnected { player_id } => {
                if !self.players.contains_key(player_id) {
                    return false;
                }
            }
            PlayerAttack { player_id, enemy_id } => {
                if !self.players.contains_key(player_id)
                    || !self.players.contains_key(enemy_id)
                {
                    return false;
                }
                if self.active_player_id != *player_id {
                    return false;
                }
            }
        }
        true
    }

    pub fn trigger(&mut self, valid_event: &GameEvent) {
        use GameEvent::*;
        match valid_event {
            BeginGame { goes_first } => {
                self.active_player_id = *goes_first;
                self.stage = Stage::InGame;
            }
            EndGame { reason: _ } => {
                self.stage = Stage::Ended;
            }
            PlayerJoined { player_id, player } => {
                self.players.insert(*player_id, player.clone());
            }
            PlayerDisconnected { player_id } => {
                self.players.remove(player_id);
            }
            PlayerAttack { player_id, enemy_id } => {
                let player_attack =
                    self.get_player_stats(player_id).unwrap().attack;
                let mut enemy = self.get_player(enemy_id).unwrap();

                enemy.base_stats.health -=
                    player_attack - enemy.base_stats.defense;

                self.active_player_id = *enemy_id;
            }
        }
    }
    pub fn get_player_stats(
        &self,
        player_id: &PlayerId,
    ) -> Option<BaseStats> {
        if let Some(player) = self.players.get(&player_id) {
            return Some(player.base_stats.clone());
        }
        None
    }

    pub fn get_player(
        &mut self,
        player_id: &PlayerId,
    ) -> Option<&mut Player> {
        if let Some(player) = self.players.get_mut(player_id) {
            return Some(player);
        }
        None
    }

    pub fn determine_winner(&self) -> Option<PlayerId> {
        let loser_id = self
            .players
            .iter()
            .find(|(_, _player)| _player.base_stats.health <= 0)
            .unwrap()
            .0;
        let winner_id = self
            .players
            .keys()
            .find(|player_id| *player_id != loser_id)
            .unwrap();

        Some(*winner_id)
    }
}
