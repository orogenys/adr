use super::Render;

pub struct ThemeToggleComponent;

impl Render for ThemeToggleComponent {
    fn render(&self) -> String {
        "<button class=\"theme-toggle\" type=\"button\" data-theme-toggle>Theme</button>"
            .to_string()
    }
}
