use std::collections;

#[derive(bon::Builder, Debug, Default)]
pub struct Inputs {
    pub key_pressed: collections::HashSet<&'static str>,
    pub key_released: collections::HashSet<&'static str>,
    pub mouse_pressed: (bool, bool),
    pub mouse_released: (bool, bool),
    pub mouse_delta: (f32, f32),
    pub request_quit: bool,
    pub toggle_grab: bool,
}

impl Inputs {
    pub fn key_press(&self, name: &str) -> bool {
        self.key_pressed.contains(name)
    }

    pub fn key_release(&self, name: &str) -> bool {
        self.key_released.contains(name)
    }

    pub fn consume_key_press(&mut self, name: &str) -> bool {
        self.key_pressed.remove(name)
    }

    pub fn consume_key_release(&mut self, name: &str) -> bool {
        self.key_released.remove(name)
    }

    pub fn consume_mouse_delta(&mut self) -> (f32, f32) {
        let delta = self.mouse_delta;
        self.mouse_delta = Default::default();
        delta
    }

    pub fn consume_mouse_left_press(&mut self) -> bool {
        let (click, _) = &mut self.mouse_pressed;
        let out = *click;
        *click = false;
        out
    }

    pub fn consume_mouse_right_press(&mut self) -> bool {
        let (_, click) = &mut self.mouse_pressed;
        let out = *click;
        *click = false;
        out
    }

    pub fn consume_mouse_left_release(&mut self) -> bool {
        let (click, _) = &mut self.mouse_released;
        let out = *click;
        *click = false;
        out
    }

    pub fn consume_mouse_right_release(&mut self) -> bool {
        let (_, click) = &mut self.mouse_released;
        let out = *click;
        *click = false;
        out
    }
}
