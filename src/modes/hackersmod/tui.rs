use dialoguer::{theme::ColorfulTheme, Select};

pub enum GhostAction {
    LaunchPhishing,
    Exit,
}

pub fn show_main_menu() -> GhostAction {
    let options = &[
        "🔒 Launch Security Training Simulation",  // Changed label to be more ethical
        "❌ Exit"
    ];
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Security Training Toolkit")
        .default(0)
        .items(options)
        .interact()
        .unwrap();

    match selection {
        0 => GhostAction::LaunchPhishing,
        _ => GhostAction::Exit,
    }
}