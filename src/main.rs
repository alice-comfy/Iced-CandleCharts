use chrono::{DateTime, Utc, TimeZone};
use iced::widget::canvas::{self, Canvas, Cursor, Frame, Path, Stroke, Text, Program, event, Event, Geometry};
use iced::{Color, Rectangle, Theme, Element, Length, Settings, Sandbox};
use rand::Rng;

// Candle data structure
#[derive(Debug, Clone)]
pub struct Candle {
    pub open: f64,
    pub low: f64,
    pub high: f64,
    pub close: f64,
    pub volume: Option<f64>,
    pub time: DateTime<Utc>,
}

// State of the chart, must implement Default
#[derive(Debug, Clone)]
pub struct CandleChartState {
    middle_pressed: bool,
    price_scale: f64,
    time_scale: f64,
}

impl Default for CandleChartState {
    fn default() -> Self {
        Self {
            middle_pressed: false,
            price_scale: 1.0,
            time_scale: 1.0,
        }
    }
}

// The CandleChart implements Program
#[derive(Debug, Clone)]
pub struct CandleChart {
    pub candles: Vec<Candle>,
}

impl CandleChart {
    pub fn new(candles: Vec<Candle>) -> Self {
        Self { candles }
    }
}

// Implement Program for CandleChart
impl<Message> Program<Message, Theme> for CandleChart {
    type State = CandleChartState;

    fn draw(
        &self,
        state: &Self::State,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(bounds.size());

        if self.candles.is_empty() {
            return vec![frame.into_geometry()];
        }

        let (min_price, max_price) = self.candles.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), c| {
            (f64::min(min, c.low), f64::max(max, c.high))
        });

        let height = bounds.height as f64;
        let width = bounds.width as f64;

        let candle_count = self.candles.len() as f64;
        let scaled_candle_count = candle_count * state.time_scale;
        let candle_spacing = width / scaled_candle_count.max(1.0);
        let candle_width = candle_spacing * 0.7;

        let price_range = (max_price - min_price) / state.price_scale;
        let mid_price = (max_price + min_price) / 2.0;
        let scaled_min_price = mid_price - price_range / 2.0;
        let scaled_max_price = mid_price + price_range / 2.0;

        let price_to_y = |p: f64| {
            let norm = (p - scaled_min_price) / (scaled_max_price - scaled_min_price + f64::EPSILON);
            height - norm * height
        };

        // Draw background
        frame.fill_rectangle(
            iced::Point::new(0.0, 0.0),
            iced::Size::new(bounds.width, bounds.height),
            Color::from_rgb8(240, 240, 240),
        );

        // Draw candles
        for (i, candle) in self.candles.iter().enumerate() {
            let x_center = i as f64 * candle_spacing + candle_spacing / 2.0;

            let high_y = price_to_y(candle.high);
            let low_y = price_to_y(candle.low);

            let stick_path = Path::line(
                iced::Point::new(x_center as f32, high_y as f32),
                iced::Point::new(x_center as f32, low_y as f32),
            );
            frame.stroke(&stick_path, Stroke::default().with_color(Color::BLACK).with_width(1.0));

            let open_y = price_to_y(candle.open);
            let close_y = price_to_y(candle.close);

            let (top, bottom) = if close_y < open_y {
                (close_y, open_y)
            } else {
                (open_y, close_y)
            };

            let body_color = if candle.close > candle.open {
                Color::from_rgb(0.0, 0.7, 0.0) // green
            } else {
                Color::from_rgb(0.7, 0.0, 0.0) // red
            };

            frame.fill_rectangle(
                iced::Point::new((x_center - candle_width / 2.0) as f32, top as f32),
                iced::Size::new(candle_width as f32, (bottom - top) as f32),
                body_color,
            );
        }

        // Vertical price labels
        let num_price_labels = 5;
        for j in 0..=num_price_labels {
            let label_price = scaled_min_price + (j as f64 / num_price_labels as f64) * (scaled_max_price - scaled_min_price);
            let y_pos = price_to_y(label_price);
            let mut text = Text {
                content: format!("{:.2}", label_price),
                position: iced::Point::new(5.0, y_pos as f32),
                color: Color::BLACK,
                size: 14.0,
                ..Text::default()
            };
            text.position.y -= text.size / 2.0;
            frame.fill_text(text);

            // Horizontal grid line
            let grid_line = Path::line(
                iced::Point::new(0.0, y_pos as f32),
                iced::Point::new(width as f32, y_pos as f32),
            );
            frame.stroke(&grid_line, Stroke::default().with_color(Color::from_rgb8(200,200,200)).with_width(1.0));
        }

        // Horizontal time labels
        let num_time_labels = 5;
        if !self.candles.is_empty() {
            for k in 0..=num_time_labels {
                let index = ((k as f64 / num_time_labels as f64) * (self.candles.len() as f64 - 1.0)) as usize;
                if let Some(candle) = self.candles.get(index) {
                    let x_center = index as f64 * candle_spacing + candle_spacing / 2.0;
                    let time_str = candle.time.format("%Y-%m-%d %H:%M").to_string();
                    let mut text = Text {
                        content: time_str,
                        position: iced::Point::new(x_center as f32, (height - 20.0) as f32),
                        color: Color::BLACK,
                        size: 14.0,
                        ..Text::default()
                    };
                    text.position.x -= (text.content.len() as f32 * text.size * 0.3) / 2.0;
                    frame.fill_text(text);

                    // Vertical grid line
                    let grid_line = Path::line(
                        iced::Point::new(x_center as f32, 0.0),
                        iced::Point::new(x_center as f32, height as f32),
                    );
                    frame.stroke(&grid_line, Stroke::default().with_color(Color::from_rgb8(220,220,220)).with_width(1.0));
                }
            }
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<Message>) {
        match event {
            Event::Mouse(mouse_event) => {
                match mouse_event {
                    iced::mouse::Event::ButtonPressed(iced::mouse::Button::Middle) => {
                        state.middle_pressed = true;
                        (event::Status::Captured, None)
                    }
                    iced::mouse::Event::ButtonReleased(iced::mouse::Button::Middle) => {
                        state.middle_pressed = false;
                        (event::Status::Captured, None)
                    }
                    iced::mouse::Event::WheelScrolled { delta } => {
                        if state.middle_pressed {
                            let scroll_amount = match delta {
                                iced::mouse::ScrollDelta::Lines { y, .. } => y,
                                iced::mouse::ScrollDelta::Pixels { y, .. } => y / 50.0,
                            };

                            let zoom_factor = 1.0 + (scroll_amount as f64 * 0.1);
                            state.price_scale *= zoom_factor;
                            state.time_scale *= zoom_factor;

                            // Clamp scales
                            if state.price_scale < 0.1 { state.price_scale = 0.1; }
                            if state.time_scale < 0.1 { state.time_scale = 0.1; }
                            if state.price_scale > 10.0 { state.price_scale = 10.0; }
                            if state.time_scale > 10.0 { state.time_scale = 10.0; }

                            (event::Status::Captured, None)
                        } else {
                            (event::Status::Ignored, None)
                        }
                    }
                    _ => (event::Status::Ignored, None),
                }
            }
            _ => (event::Status::Ignored, None),
        }
    }
}

pub struct CandleChartApp {
    candles: Vec<Candle>,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl Sandbox for CandleChartApp {
    type Message = Message;

    fn new() -> Self {
        let start = Utc.with_ymd_and_hms(2022, 10, 1, 0, 0, 0).unwrap();
        let mut rng = rand::thread_rng();
        let mut candles = Vec::new();

        let mut last_close = 100.0;
        for i in 0..24 {
            let time = start + chrono::Duration::hours(i);
            let open = last_close;
            let high = open + (rng.gen::<f64>() * 5.0);
            let low = open - (rng.gen::<f64>() * 5.0);
            let close = low + (rng.gen::<f64>() * (high - low));
            last_close = close;
            candles.push(Candle {
                open,
                high,
                low,
                close,
                volume: Some((rng.gen::<f64>() * 1000.0).abs()),
                time,
            });
        }

        CandleChartApp { candles }
    }

    fn title(&self) -> String {
        String::from("Candle Chart Demo")
    }

    fn update(&mut self, _message: Message) {
        // no messages
    }

    fn view(&self) -> Element<Message> {
        Canvas::new(CandleChart::new(self.candles.clone()))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn main() {
    CandleChartApp::run(Settings::default()).unwrap();
}
