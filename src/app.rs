use anyhow::Result;
use ratatui::Frame;

use crate::dmi::DMI;

#[non_exhaustive]
#[derive(Debug)]
pub enum ActivePopup {
    Help,
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub dmi: DMI,
}

impl App {
    pub fn new() -> Result<Self> {
        let dmi = DMI::new()?;
        Ok(Self { running: true, dmi })
    }

    pub fn render(&mut self, frame: &mut Frame) {
        self.dmi.render(frame);
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
