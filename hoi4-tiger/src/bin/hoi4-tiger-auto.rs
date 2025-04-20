use anyhow::Result;
use hoi4_tiger::GAME_CONSTS;
use tiger_bin_shared::auto;

fn main() -> Result<()> {
    auto(GAME_CONSTS)
}
