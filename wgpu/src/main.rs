pub mod application;
pub mod game;
pub mod render;

fn main() -> anyhow::Result<()> {
    application::run::<game::GameState>()?;
    Ok(())
}
