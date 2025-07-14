// sitegen.rs
use dialoguer::{theme::ColorfulTheme, Select};

pub fn generate_html(
    brand: &str, 
    business: &str, 
    logo_url: &str, 
    theme: &str
) -> String {
    let (bg, text_color, accent, shadow) = match theme {
        "blue_cyberpunk" => ("#0a0f2c", "#00ffff", "#00aaff", "#00f0ff"),
        "amber_terminal" => ("#1b140b", "#ffbf00", "#ffcc00", "#ffea00"),
        _ => ("#000000", "#00ff00", "#00dd00", "#00ff00"), 
    };

    let logo_html = if !logo_url.trim().is_empty() {
        format!(
            r#"<img src="{}" alt="logo" style="width:80px;margin-bottom:20px;">"#,
            logo_url
        )
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{brand} - Security Training</title>
    <style>
        body {{
            background-color: {bg};
            color: {text};
            font-family: 'Courier New', monospace;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
            margin: 0;
        }}
        .terminal-box {{
            border: 2px solid {accent};
            padding: 40px;
            border-radius: 10px;
            width: 400px;
            background: {bg};
            box-shadow: 0 0 20px {shadow};
            text-align: center;
        }}
        input {{
            background: {bg};
            color: {text};
            border: 1px solid {accent};
            padding: 10px;
            margin: 10px 0;
            width: 100%;
        }}
        button {{
            background-color: {accent};
            color: {bg};
            padding: 10px;
            border: none;
            width: 100%;
            margin-top: 10px;
            font-weight: bold;
        }}
        .warning {{
            margin-top: 20px;
            font-size: 12px;
            color: yellow;
        }}
    </style>
</head>
<body>
    <div class="terminal-box">
        {logo}
        <h2>{brand}</h2>
        <p><i>({business})</i></p>
        <form method="POST" action="/login">
            <input type="text" name="username" placeholder="Username" autocomplete="off">
            <input type="password" name="password" placeholder="Password" autocomplete="off">
            <button type="submit">ACCESS</button>
        </form>
        <div class="warning">
            ⚠️ SECURITY TRAINING: This is a simulated login page for educational purposes only.
            Do not enter real credentials.
        </div>
    </div>
</body>
</html>"#,
        brand = brand,
        business = business,
        logo = logo_html,
        bg = bg,
        text = text_color,
        accent = accent,
        shadow = shadow
    )
}

pub fn select_template() -> String {
    let templates = vec![
        "retro_green",
        "blue_cyberpunk",
        "amber_terminal",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a theme for security training")
        .items(&templates)
        .default(0)
        .interact()
        .unwrap();

    templates[selection].to_string()
}