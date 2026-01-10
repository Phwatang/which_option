use std::iter;
use std::ops::RangeInclusive;
use iced::Element;
use iced::widget::{column, text};
use iced::widget::canvas::{Cache, Frame, Geometry};
use iced::Size;
use plotters_iced2::{Renderer};
use plotters_iced2::{Chart, ChartWidget, DrawingBackend, ChartBuilder};
use iced::Center;

#[derive(Debug, Clone, Copy)]
pub enum PayoffChartMessage{}

/// Determines number of datapoints computed for all charts
const CHART_RESOLUTION: i32 = 501;

/// Determines chart title text size
const CHART_TITLE_SIZE: u32 = 25;

/// Font used for any text rendered by plotters
// Imported from enabling iced's "fira-sans" feature
const CHART_FONT_NAME: &str = "Fira Sans";

/// Chart widget to handle drawing a single payoff chart.
/// Support drawing an ROI graph or a nominal return graph.
pub struct PayoffChart {
    cache: Cache,
    /// Main payoff function to plot
    func: Box<dyn Fn(f64) -> f64>,
    /// The height of the "benchmark" line. For an ROI graph this would be 1.
    benchmark: f64,
    /// x axis range of the graph
    x_range: RangeInclusive<f64>,
    /// y axis range of the graph
    y_range: RangeInclusive<f64>,
    /// x value of where to place vertical crosshair line
    x_vert: Option<f64>,
    /// Chart title
    title: String,
    /// x-axis title
    title_x: String,
    /// Lines series labels
    labels: [String; 2]
}
impl Default for PayoffChart {
    fn default() -> Self {
        Self {
            cache: Cache::new(),
            func: Box::new(|x| x),
            benchmark: 1.0,
            x_range: 0.0f64..=10.0f64,
            y_range: 0.0f64..=10.0f64,
            x_vert: None,
            title: String::from("Title"),
            title_x: String::from("X-Axis Title"),
            labels: [String::from("Line 1"), String::from("Line 2")]
        }
    }
}
impl PayoffChart {
    pub fn view(&self) -> Element<'_, PayoffChartMessage> {
        column![
            text!("{}", self.title).size(CHART_TITLE_SIZE),
            ChartWidget::new(self),
            text!("{}", self.title_x).size(CHART_TITLE_SIZE - 10),
        ].align_x(Center)
        .into()
    }

    /// Create chart for showing ROI (return on investment)
    pub fn new_roi_chart(chart_title: String, x_axis_title: String) -> Self {
        return Self {
            title: chart_title,
            title_x: x_axis_title,
            benchmark: 1.0,
            labels: [String::from("Exit ROI"), String::from("Entry ROI")],
            ..Default::default()
        }
    }

    /// Create chart for showing pure nominal amounts
    pub fn new_nominal_chart(chart_title: String, x_axis_title: String) -> Self {
        return Self {
            title: chart_title,
            title_x: x_axis_title,
            labels: [String::from("Exit Price"), String::from("Entry Price")],
            ..Default::default()
        }
    }

    /// Sets the height of the benchmark line
    pub fn set_benchmark_height(&mut self, height: f64) -> &mut Self {
        self.benchmark = height;
        self.cache.clear();
        return self;
    }

    /// Sets the range of x-axis values the chart will cover
    pub fn set_xrange(&mut self, x_range: RangeInclusive<f64>) -> &mut Self {
        self.x_range = x_range;
        self.cache.clear();
        return self;
    }

    /// Sets the (minimum) range of y-axis values the chart will cover
    pub fn set_yrange(&mut self, y_range: RangeInclusive<f64>) -> &mut Self {
        self.y_range = y_range;
        self.cache.clear();
        return self;
    }

    /// Sets the payoff function the chart will draw
    pub fn set_func(&mut self, func: Box<dyn Fn(f64) -> f64>) -> &mut Self {
        self.func = func;
        self.cache.clear();
        return self;
    }
    
    /// Sets the x-value of the crosshair line
    pub fn set_x_vert(&mut self, x: f64) -> &mut Self {
        self.x_vert = Some(x);
        self.cache.clear();
        return self;
    }
}
impl Chart<PayoffChartMessage> for PayoffChart {
    type State = ();

    #[inline]
    fn draw<R: Renderer, F: Fn(&mut Frame)>(
        &self,
        renderer: &R,
        bounds: Size,
        draw_fn: F,
    ) -> Geometry {
        renderer.draw_cache(&self.cache, bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(&self, _: &Self::State, mut chart: ChartBuilder<DB>) {
        use plotters::prelude::*;
        const BLUE_LINE_COLOR: RGBColor = RGBColor(0, 175, 255);
        const RED_LINE_COLOR: RGBColor = RGBColor(220, 20, 20);
        const BLACK_LINE_COLOR: RGBColor = RGBColor(0, 0, 0);

        let start = *self.x_range.start();
        let end = *self.x_range.end();
        let x_linspace: Vec<f64> = (0..CHART_RESOLUTION).into_iter()
            .map(|x| start + x as f64*((end-start)/((CHART_RESOLUTION-1) as f64)) )
            .collect();

        // Ensure y range of the graph is atleast self.y_range (or wider if needed)
        let func_max = x_linspace.iter()
            .map(|&x| (self.func)(x))
            .reduce(f64::max)
            .unwrap_or(0.0);
        let mut y_range = 0.0..=func_max;
        y_range = *y_range.start()..=y_range.end().max(*self.y_range.end());

        let x_range_exclusive = *self.x_range.start()..*self.x_range.end();
        let y_range_exclusive = *y_range.start()..*y_range.end();
        let mut chart = chart
            .x_label_area_size(20)
            .y_label_area_size(40)
            .margin(10)
            .build_cartesian_2d(x_range_exclusive, y_range_exclusive)
            .expect("failed to build chart");

        // General chart formatting
        chart
            .configure_mesh()
            .label_style((CHART_FONT_NAME).into_font())
            .bold_line_style(plotters::style::colors::BLUE.mix(0.1))
            .light_line_style(plotters::style::colors::BLUE.mix(0.05))
            .axis_style(ShapeStyle::from(plotters::style::colors::BLUE.mix(0.45)).stroke_width(1))
            .y_labels(10)
            .y_label_formatter(&|y: &f64| format!("{:.1}", y))
            .draw()
            .expect("failed to draw chart mesh");

        // Draw the function given at self.func
        chart.draw_series(
                AreaSeries::new(
                    x_linspace.iter().map(|&x| (x, (self.func)(x))),
                    0.0,
                    BLUE_LINE_COLOR.mix(0.175),
                )
                .border_style(ShapeStyle::from(BLUE_LINE_COLOR).stroke_width(2)),
            ).expect("failed to draw chart data")
            // Empty spaces to act as margin
            .label(format!("{}", self.labels[0].to_owned()))
            // y+5 is to lower the legend-line to be inline with the label
            .legend(|(x, y)| PathElement::new(vec![(x, y+5), (x + 20, y+5)], BLUE_LINE_COLOR));

        // Draw profit benchmark line
        chart.draw_series(
                AreaSeries::new(
                    x_linspace.iter().map(|&x| (x, self.benchmark)),
                    0.0,
                    RED_LINE_COLOR.mix(0.175),
                )
                .border_style(ShapeStyle::from(RED_LINE_COLOR).stroke_width(2)),
            ).expect("failed to draw chart data")
            // Empty spaces to act as margin
            .label(format!("{}\n({:.2})", self.labels[1].to_owned(), self.benchmark))
            // y+5 is to lower the legend-line to be inline with the label
            .legend(|(x, y)| PathElement::new(vec![(x, y+5), (x + 20, y+5)], RED_LINE_COLOR));
        
        // Invisible filler line.
        // Only being used to we can add an invisible line to the lineseries label box. For some reason
        // the \n character printed from the series label before this is not respected during margin calculations
        // when drawing the border box.
        chart.draw_series(
                AreaSeries::new(
                    x_linspace.iter().map(|&x| (x, x)),
                    0.0,
                    BLACK_LINE_COLOR.mix(0.0)
                )
                .border_style(ShapeStyle::from(RED_LINE_COLOR).stroke_width(0))
            )
            .expect("failed to draw chart data")
            .label(" ");

        // Draw vertical crosshair line (if valid)
        if let Some(x_vert) = self.x_vert {
            let val = (self.func)(x_vert);
            if val.is_nan() {
                return;
            }
            chart.draw_series(
                LineSeries::new(
                    [(x_vert, 0.0), (x_vert, f64::MAX)].iter().map(|&x| x),
                    BLACK_LINE_COLOR
                )
            ).expect("failed to draw chart data");
            // Highlight where vertical line intersects main function
            chart.draw_series(PointSeries::of_element(
                iter::once((x_vert, val)),
                5,
                ShapeStyle::from(&RED).filled(),
                &|coord, size, style| {
                    EmptyElement::at(coord)
                    + Circle::new((0, 0), size, style)
                    + Text::new(format!("({:.3}, {:.3})", coord.0, coord.1), (0, 15), (CHART_FONT_NAME, 15))
                },
            )).expect("failed to draw chart data");
        }

        // Draw line legends
        chart.configure_series_labels()
            .border_style(BLACK)
            .label_font((CHART_FONT_NAME, 15))
            .draw()
            .expect("failed to draw line labels");
    }
}
