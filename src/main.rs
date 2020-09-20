use anyhow::Result;
use bitwarden_rofi::app::App;

fn main() -> Result<()> {
    App::new()?.show()
}
