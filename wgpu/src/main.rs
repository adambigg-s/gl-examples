use crate::engine::application;

pub mod engine;
pub mod game;

fn main() -> anyhow::Result<()> {
    application::run::<game::GameState>()?;
    Ok(())
}
