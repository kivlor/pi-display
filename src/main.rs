use std::time::{Duration, Instant};

use chrono::{Local, Timelike};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};
use serde::Deserialize;
use tui_big_text::{BigText, PixelSize};

// Brisbane, Australia coordinates
const LATITUDE: f64 = -27.4698;
const LONGITUDE: f64 = 153.0251;
const WEATHER_UPDATE_INTERVAL: Duration = Duration::from_secs(30 * 60); // 30 minutes

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    current: CurrentWeather,
    daily: DailyWeather,
}

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    temperature_2m: f64,
    weather_code: u8,
}

#[derive(Debug, Deserialize)]
struct DailyWeather {
    time: Vec<String>,
    weather_code: Vec<u8>,
    temperature_2m_max: Vec<f64>,
    temperature_2m_min: Vec<f64>,
}

#[derive(Debug, Clone)]
struct WeatherData {
    current_temp: f64,
    current_condition: String,
    forecast: Vec<ForecastDay>,
}

#[derive(Debug, Clone)]
struct ForecastDay {
    day_name: String,
    high: f64,
    low: f64,
    condition: String,
}

fn weather_code_to_condition(code: u8) -> &'static str {
    match code {
        0 => "Clear",
        1..=3 => "Cloudy",
        45..=48 => "Fog",
        51..=67 => "Rain",
        71..=77 => "Snow",
        80..=82 => "Showers",
        95..=99 => "Storm",
        _ => "Unknown",
    }
}

fn fetch_weather() -> Option<WeatherData> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?\
        latitude={}&longitude={}&\
        current=temperature_2m,weather_code&\
        daily=weather_code,temperature_2m_max,temperature_2m_min&\
        timezone=Australia/Brisbane&\
        forecast_days=8",
        LATITUDE, LONGITUDE
    );

    let response = reqwest::blocking::get(&url).ok()?;
    let data: OpenMeteoResponse = response.json().ok()?;

    let forecast: Vec<ForecastDay> = data
        .daily
        .time
        .iter()
        .skip(1) // Skip today, show next 7 days
        .take(7)
        .enumerate()
        .map(|(i, date)| {
            let idx = i + 1; // Offset for skipped today
            ForecastDay {
                day_name: chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                    .map(|d| d.format("%a").to_string())
                    .unwrap_or_else(|_| "???".to_string()),
                high: data.daily.temperature_2m_max[idx],
                low: data.daily.temperature_2m_min[idx],
                condition: weather_code_to_condition(data.daily.weather_code[idx]).to_string(),
            }
        })
        .collect();

    Some(WeatherData {
        current_temp: data.current.temperature_2m,
        current_condition: weather_code_to_condition(data.current.weather_code).to_string(),
        forecast,
    })
}

fn main() -> std::io::Result<()> {
    ratatui::run(|terminal| {
        // Clear the entire screen on startup
        terminal.clear()?;

        // Force a full redraw by clearing the back buffer
        for _ in 0..2 {
            terminal.draw(|frame| {
                frame.render_widget(Clear, frame.area());
            })?;
        }

        // Fetch weather on startup
        let mut weather: Option<WeatherData> = fetch_weather();
        let mut last_weather_fetch = Instant::now();

        loop {
            let now = Local::now();
            // Blink every half second based on milliseconds
            let show_colon = (now.timestamp_millis() / 500) % 2 == 0;

            terminal.draw(|frame| render(frame, show_colon, weather.as_ref()))?;

            // Poll for events with 500ms timeout to update the blink
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(key) = event::read()? {
                    if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                        break Ok(());
                    }
                }
            }

            // Refresh weather every 30 minutes
            if last_weather_fetch.elapsed() >= WEATHER_UPDATE_INTERVAL {
                if let Some(new_weather) = fetch_weather() {
                    weather = Some(new_weather);
                }
                last_weather_fetch = Instant::now();
            }
        }
    })
}

fn render(frame: &mut Frame, show_colon: bool, weather: Option<&WeatherData>) {
    // Clear the entire frame area
    frame.render_widget(Clear, frame.area());

    // Split screen: 2/3 for time/date, 1/3 for weather
    let [time_section, weather_section] =
        Layout::horizontal([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)]).areas(frame.area());

    // Render time/date section
    render_time_date(frame, time_section, show_colon);

    // Render weather section
    render_weather(frame, weather_section, weather);
}

fn render_time_date(frame: &mut Frame, area: ratatui::layout::Rect, show_colon: bool) {
    // Add border around time/date panel
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let now = Local::now();

    // Format time with blinking colon
    let separator = if show_colon { ":" } else { " " };
    let time_str = format!("{:02}{}{:02}", now.hour(), separator, now.minute());

    // Format date as "Saturday, January 4"
    let date_str = now.format("%A, %B %-d").to_string();

    // Create vertical layout: time on top, date below
    let [time_area, _, date_area] = Layout::vertical([
        Constraint::Length(8), // BigText height
        Constraint::Length(1), // Spacer
        Constraint::Length(2), // Date text height
    ])
    .flex(Flex::Center)
    .areas(inner);

    // Render big time display centered
    let time_widget = BigText::builder()
        .pixel_size(PixelSize::Full)
        .style(Style::default().fg(Color::White))
        .lines(vec![time_str.into()])
        .centered()
        .build();

    frame.render_widget(time_widget, time_area);

    // Render date as regular text (centered)
    let date_widget = Paragraph::new(date_str)
        .style(Style::default().fg(Color::Gray))
        .centered();

    frame.render_widget(date_widget, date_area);
}

fn render_weather(frame: &mut Frame, area: ratatui::layout::Rect, weather: Option<&WeatherData>) {
    // Add border around weather panel (no title)
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(weather) = weather else {
        let loading = Paragraph::new("Loading...")
            .style(Style::default().fg(Color::Gray))
            .centered();
        frame.render_widget(loading, inner);
        return;
    };

    // Split weather area: current, condition, forecast (with spacing)
    let [current_area, condition_area, _, forecast_area] = Layout::vertical([
        Constraint::Length(4), // Current temp
        Constraint::Length(1), // Current condition
        Constraint::Length(1), // Spacer
        Constraint::Length(5), // 5 day forecast
    ])
    .flex(Flex::Center)
    .areas(inner);

    // Current weather as big text
    let current_text = format!("{}c", weather.current_temp.round() as i32);
    let current_widget = BigText::builder()
        .pixel_size(PixelSize::HalfHeight)
        .style(Style::default().fg(Color::Gray))
        .lines(vec![current_text.into()])
        .centered()
        .build();

    frame.render_widget(current_widget, current_area);

    let condition_text = format!("{}", weather.current_condition);
    let condition_widget = Paragraph::new(condition_text)
        .style(Style::default().fg(Color::Gray))
        .centered();

    frame.render_widget(condition_widget, condition_area);

    // Forecast
    let forecast_lines: Vec<String> = weather
        .forecast
        .iter()
        .map(|day| {
            format!(
                "{} {}c/{}c {}",
                day.day_name,
                day.low.round() as i32,
                day.high.round() as i32,
                day.condition
            )
        })
        .collect();

    let forecast_text = forecast_lines.join("\n");
    let forecast_widget = Paragraph::new(forecast_text)
        .style(Style::default().fg(Color::Gray))
        .centered();
    frame.render_widget(forecast_widget, forecast_area);
}
