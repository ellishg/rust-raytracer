
use super::utils::clamp;

#[derive(Debug, Copy, Clone)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    pub fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color::rgba(r, g, b, 1.0)
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        let r = clamp(r, 0.0, 1.0);
        let g = clamp(g, 0.0, 1.0);
        let b = clamp(b, 0.0, 1.0);
        let a = clamp(a, 0.0, 1.0);
        Color { r, g, b, a }
    }

    pub fn get_rgb(&self) -> (u8, u8, u8) {
        (
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
        )
    }
}

impl std::ops::Add for Color {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        // TODO: How should we add colors?
        Color::rgba(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
            self.a + rhs.a,
        )
    }
}

impl std::ops::Mul for Color {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Color::rgba(
            self.r * rhs.r,
            self.g * rhs.g,
            self.b * rhs.b,
            self.a * rhs.a,
        )
    }
}
impl std::ops::Mul<f32> for Color {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        // TODO: How should we multiply colors?
        Color::rgba(self.r * rhs, self.g * rhs, self.b * rhs, self.a * rhs)
    }
}
impl std::ops::Mul<Color> for f32 {
    type Output = Color;
    fn mul(self, rhs: Color) -> Color {
        rhs * self
    }
}