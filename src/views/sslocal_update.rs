use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Margin},
    style::{palette::tailwind::*, Style, Styled, Stylize},
    text::Text,
    widgets::{
        HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
};

use super::{
    messagebox::{MessageBoxLayer, YesNoMessageBoxLayer},
    sslocal_download::SSLocalDownloadLayer,
};
use crate::{
    sslocal::{LatestRelease, SSLocal},
    Layer, Show,
};

pub struct SSLocalUpdateLayer {
    exit: bool,
    latest: LatestRelease,
    name_longest_len: u16,
    size_str_vec: Vec<String>,
    size_longest_len: u16,
    table_state: TableState,
    selected_style: Style,
    row_style: [Style; 2],
    scroll_state: ScrollbarState,
    pub result: Option<SSLocal>,
}

impl SSLocalUpdateLayer {
    pub fn new(latest: LatestRelease) -> Self {
        let name_longest_len = latest
            .assets
            .iter()
            .map(|asset| asset.name.len())
            .max()
            .unwrap_or(0) as u16;
        let size_str_vec: Vec<String> = latest
            .assets
            .iter()
            .map(|asset| format!("{:.2} MB", asset.size as f32 / 1024.0 / 1024.0))
            .collect();
        let size_longest_len = size_str_vec.iter().map(|s| s.len()).max().unwrap_or(0) as u16;

        let selected_style = Style::default().fg(BLACK).bg(INDIGO.c400);
        let row_style = [
            Style::default().fg(WHITE).bg(GRAY.c950),
            Style::default().fg(WHITE).bg(GRAY.c900),
        ];
        let scroll_state = ScrollbarState::new((latest.assets.len() - 1) * 3);

        Self {
            exit: false,
            latest,
            name_longest_len,
            size_str_vec,
            size_longest_len,
            table_state: TableState::default().with_selected(0),
            selected_style,
            row_style,
            scroll_state,
            result: None,
        }
    }
}

impl Layer for SSLocalUpdateLayer {
    fn view(&mut self, frame: &mut ratatui::Frame) {
        let [table_layout, footer_layout] =
            Layout::vertical([Constraint::Percentage(100), Constraint::Length(1)])
                .areas(frame.area());

        let footer = Paragraph::new("Exit (Esc) | Select (Enter) | Up (↑) | Down (↓)")
            .white()
            .on_cyan()
            .centered();
        frame.render_widget(footer, footer_layout);

        let header = Row::new(["Name", "Size", "Download Link"])
            .white()
            .on_blue();
        let rows = self.latest.assets.iter().enumerate().map(|(i, asset)| {
            Row::new([
                format!("\n{}\n", asset.name),
                format!("\n{}\n", self.size_str_vec[i]),
                format!("\n{}\n", asset.browser_download_url),
            ])
            .set_style(self.row_style[i % 2])
            .height(3)
        });
        let table = Table::new(
            rows,
            [
                Constraint::Length(self.name_longest_len + 1),
                Constraint::Length(self.size_longest_len + 1),
                Constraint::Percentage(100),
            ],
        )
        .header(header)
        .highlight_style(self.selected_style)
        .highlight_symbol(Text::from(vec!["".into(), " █ ".into(), "".into()]))
        .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(table, table_layout, &mut self.table_state);

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
        frame.render_stateful_widget(
            scrollbar,
            table_layout.inner(Margin {
                horizontal: 0,
                vertical: 1,
            }),
            &mut self.scroll_state,
        );
    }

    fn before_show(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn update(&mut self, event: Option<ratatui::crossterm::event::Event>) -> std::io::Result<()> {
        if let Some(event) = event {
            if let Event::Key(key_event) = event {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => self.exit = true,
                        KeyCode::Up => {
                            self.table_state.select_previous();
                            if let Some(i) = self.table_state.selected() {
                                self.scroll_state = self.scroll_state.position(i * 3);
                            }
                        }
                        KeyCode::Down => {
                            self.table_state.select_next();
                            if let Some(i) = self.table_state.selected() {
                                self.scroll_state = self.scroll_state.position(i * 3);
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(i) = self.table_state.selected() {
                                if self.latest.assets[i].name.ends_with(".zip")
                                    || self.latest.assets[i].name.ends_with(".tar.xz")
                                {
                                    let yes_no = YesNoMessageBoxLayer::new(
                                        "Info",
                                        format!("download '{}' ?", self.latest.assets[i].name),
                                    )
                                    .green()
                                    .on_gray()
                                    .show()?;
                                    if yes_no.result.is_yes() {
                                        let download = SSLocalDownloadLayer::new(
                                            self.latest.assets[i].clone(),
                                        )
                                        .show()?;
                                        if download.result.is_some() {
                                            self.result = download.result;
                                            self.exit = true;
                                        }
                                    }
                                } else {
                                    MessageBoxLayer::new("Error", "unsupported file type")
                                        .red()
                                        .on_gray()
                                        .show()?;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn close(&mut self) {
        self.exit = true;
    }

    fn is_exit(&self) -> bool {
        self.exit
    }
}
