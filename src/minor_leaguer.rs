use rand::rngs::ThreadRng;

use crate::{
    era::Era,
    player::{Hand, Player, PlayerGender},
    player_quality::PlayerQuality,
};

#[derive(Debug)]
/// When a Deadball team has a player in the farm system,only a few fields for the player are generated, and hte rest are created when a player is promoted.
/// This struct contains what is known about a player, including quality, that is used to generate the player once it is promoted.
struct MinorLeaguer<PotentialQuality: PlayerQuality> {
    potential_quality: PotentialQuality,
    name: String,
    hand: Hand,
    gender: PlayerGender,
    era: Era,
}

impl<T: PlayerQuality> MinorLeaguer<T> {
    fn upgrade(&mut self) {
        self.potential_quality = self.potential_quality.upgrade()
    }

    fn unveil(&self, thread: &mut ThreadRng) -> Player {
        let gen_player = self.potential_quality.gen_player(thread, self.era);

        Player {
            name: self.name.clone(),
            hand: self.hand,
            ..gen_player
        }
    }
}
