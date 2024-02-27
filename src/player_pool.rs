
pub enum PlayerCategory{

  Batter,
  Pitcher,
  Both


}

impl PlayerCategory{

  pub fn filter(&self, player: &Player) -> bool{ 
  
    match self{
    
      Self::Both => true,
      Self::Batter => !player.is_pitcher(),
      Self::Pitcher => player.is_pitcher
    
    
    }
  
  
  
  }

}


pub Struct PlayerPool{

  vec: Vec<Player>
  category: PlayerCategory,
  desc: String


}


impl PlayerPool{

  pub fn new(desc: &str,category: PlayerCategory) -> PlayerPool{
  
    PlayerPool{
      vec: Vec::new();
      category: category,
      desc: desc.to_string()
    
    
    }
  
  }

  pub fn add_player(&self, player: Player){
    match self.category.filter(&player){
    
      true = self.vec.push(player),
      false = panic!(format!("Tried to add an incorrect player category to a {} pool",self.category.to_string()))
    
    }
  
  }


}


