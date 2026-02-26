mod browser;
mod credits;
mod home;
mod player;
mod settings;
mod widgets;

use crate::app::{App, AppPage};
use ratatui::{backend::Backend, Frame};

pub fn draw<B: Backend>(f: &mut Frame, app: &mut App) {
    match &app.page {
        AppPage::Home        => home::draw(f, app),
        AppPage::NowPlaying  => player::draw(f, app),
        AppPage::FileBrowser => browser::draw(f, app),
        AppPage::Settings    => settings::draw(f, app),
        AppPage::Credits     => credits::draw(f, app),
    }
}
