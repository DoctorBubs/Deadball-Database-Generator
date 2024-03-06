
use crate ::player::Player;

pub enum FutureChange{

    BT(i32),
    PD(i32)

}

pub struct Future{

    change: FutureChange,
    playerID: i32


}

fn get_player_from_id(id:i32,vec: &mut Vec<Player>) -> Result<&mut Player,()>{

    for player in vec{


        if player.id == id{

            return Ok(player)

        };
    };

    Err(())

}

impl Future{


    fn activate(&self,vec: &mut Vec<Player>){
        match get_player_from_id(self.id, vec){


            Ok(player) =>{
                match self.change{


                    BT(diff) => {
                        player.bt += diff;
                        player.calc_obt(); 
                    },
                    PD(diff) => {}
                }
            },
        }
    }
        
}
