use rand::rngs::ThreadRng;

use crate::{
    era::Era,
    player::{Hand, Player, PlayerGender},
    player_quality::{BatterQuality, PitcherQuality, PlayerQuality},
};

#[derive(Debug)]
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
