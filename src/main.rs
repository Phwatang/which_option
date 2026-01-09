use std::ops::RangeInclusive;

mod blackscholes;
use blackscholes::{
    Environment, Contract, Movement,
    BlackScholes, BlackScholesROI,
    Call, Put,
};

mod custom_widgets;
use custom_widgets::{
    NumberInput, NumberInputMessage, 
    CustomSlider, CustomSliderMessage, 
    DeletableList, DeletableListMessage,
    PayoffChart, PayoffChartMessage,
};

use iced::Alignment::Center;
use iced::window::Settings;
use iced::{Element, Font, Left, Length, Subscription, Task};
use iced::widget::{Column, button, column, container, operation, pick_list, responsive, row, rule, scrollable, text, tooltip};

/// Font to be used by all text rendered with Iced
// Imported from enabling Iced's "fira-sans" feature
const FONT_NAME: &str = "Fira Sans";

/// Limits the number of decimal points the calculator will output and the amount for inputs
const MAX_DP: usize = 3;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Adjustables {
    Strike,
    Expiry,
    EndPrice,
    EndTime,
    EndVol,
}
impl std::fmt::Display for Adjustables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Strike => "Strike",
            Self::Expiry => "Expiry",
            Self::EndPrice => "Stock End Price",
            Self::EndTime => "End Time",
            Self::EndVol => "End Volatility",
        })
    }
}
impl Adjustables {
    const COUNT: usize = 5;

    pub fn everything() -> [Self; Self::COUNT] {
        [Self::Strike,
        Self::Expiry,
        Self::EndPrice,
        Self::EndTime,
        Self::EndVol]
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum PayoffYAxis {
    ROI,
    Nominal
}
impl std::fmt::Display for PayoffYAxis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ROI => "ROI",
            Self::Nominal => "Nominal",
        })
    }
}
impl PayoffYAxis {
    const COUNT: usize = 2;

    pub fn everything() -> [Self; Self::COUNT] {
        [Self::ROI, Self::Nominal]
    }
}

pub fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init().expect("Initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    let mut window_setting = Settings::default();
    window_setting.min_size = Some((900.0, 400.0).into());

    iced::application(OptionCalculator::default, OptionCalculator::update, OptionCalculator::view)
        .antialiasing(true)
        .default_font(Font::with_name(FONT_NAME))
        .subscription(OptionCalculator::subscription)
        .window(window_setting)
        .run()
}

// #[derive(Default)]
struct OptionCalculator {
    /// Members descriptions (in order):
    ///  - bool: true if using Call contract
    ///  - Contract: The strike and expiry of the answer
    ///  - f64: Purchase price of the contract
    ///  - f64: Selling price of the contract
    ///  - f64: ROI of buying then selling the contract
    answers: (bool, Contract, f64, f64, f64),
    /// Input boxes for the starting environment
    param: [NumberInput; 6],
    /// Environment variables extracted from user numeric input
    start_env: Environment,
    /// Price movement extracted from user numeric input. Can
    /// later be adjusted by the user with numeric sliders.
    movement: Movement,
    /// Environment variables for what happens at the prediction end date.
    /// Can later be adjusted by the user with numeric sliders.
    end_env: Environment,
    /// The "optimal" contract calculated given a starting environment and prediction.
    /// Can later be adjusted by user with numeric sliders
    contract: Contract,
    /// Payoff charts to visualise returns against a single variable changing
    charts: DeletableList<
        (PayoffYAxis, Adjustables),
        PayoffChart, 
        PayoffChartMessage, 
        fn(&mut PayoffChart, PayoffChartMessage), 
        fn(&PayoffChart) -> Element<'_, PayoffChartMessage>>,
    /// Sliders to quickly vary variables of the scenario for the payoff calculation
    sliders: DeletableList<
        Adjustables,
        CustomSlider, 
        CustomSliderMessage, 
        fn(&mut CustomSlider, CustomSliderMessage), 
        fn(&CustomSlider) -> Element<'_, CustomSliderMessage>>,
    slider_add_select: Option<Adjustables>,
    chart_y_select: Option<PayoffYAxis>,
    chart_x_select: Option<Adjustables>,
    ranges: [RangeInclusive<f64>; Adjustables::COUNT],
}

impl Default for OptionCalculator {
    fn default() -> Self {
        use core::array;

        Self {
            sliders: DeletableList::new(CustomSlider::update, CustomSlider::view),
            answers: Default::default(),
            param: array::from_fn(|_| {
                let mut input = NumberInput::default().set_precision(MAX_DP);
                input.set_range(0.0..=f64::MAX);
                input
            }),
            start_env: Default::default(),
            end_env: Default::default(),
            movement: Default::default(),
            contract: Default::default(),
            charts: DeletableList::new(|_, _| {}, PayoffChart::view),
            slider_add_select: Default::default(),
            chart_y_select: Default::default(),
            chart_x_select: Default::default(),
            ranges: array::from_fn(|_| 0.0..=0.0),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Charts(DeletableListMessage<PayoffChartMessage>),
    Calculate,
    NumberInputMessage(usize, NumberInputMessage),
    Sliders(DeletableListMessage<CustomSliderMessage>),
    SliderSelect(Adjustables),
    SliderAdd,
    ChartXSelect(Adjustables),
    ChartYSelect(PayoffYAxis),
    ChartAdd,
    TabPressed,
}

impl OptionCalculator {
    /// Checks if all user parameter inputs are present and >=0
    fn extract_env_and_pred(&self) -> Option<(Environment, Movement)> {
        for input in self.param.iter() {
            if input.get_value().is_nan() || input.value_outside_range() {
                return None;
            }
        }
        return Some(
            (Environment { 
                stock: self.param[0].get_value(), 
                risk_free: self.param[2].get_value(), 
                vol: self.param[1].get_value(), 
                div_yield: self.param[3].get_value() 
            },
            Movement {
                stock: self.param[4].get_value(),
                time: self.param[5].get_value()
            })
        );
    }

    fn answer_text_block(&self) -> [String; 6] {
        let mut out: [String; 6] = Default::default();
        if self.answers.0 == true {
            out[0] = String::from("Utilising Calls");
        } else {
            out[0] = String::from("Utilising Puts");
        }
        out[1] = format!("Strike: {:.3}", self.answers.1.strike);
        out[2] = format!("Expiry: {:.3}", self.answers.1.expiry);
        out[3] = format!("Buy Price: {:.3}", self.answers.2);
        out[4] = format!("Sell Price: {:.3}", self.answers.3);
        out[5] = format!("ROI: {:.3}", self.answers.4);
        return out;
    }

    /// Creates a "reasonable" range of values the given variable should be able to take up
    fn create_default_range(&self, var: Adjustables) -> RangeInclusive<f64> {
        match var {
            Adjustables::Strike => 0.0..=2.0*self.contract.strike,
            Adjustables::Expiry => self.movement.time..=2.0*self.movement.time,
            Adjustables::EndPrice => 0.0..=2.0*self.movement.stock,
            Adjustables::EndTime => 0.0..=self.contract.expiry,
            Adjustables::EndVol => 0.0..=2.0*self.end_env.vol,
        }
    }

    /// Creates the widest range of values that is valid (or makes "sense") for the given variable
    fn create_valid_range(&self, var: Adjustables) -> RangeInclusive<f64> {
        match var {
            Adjustables::Strike => 0.0..=f64::MAX,
            Adjustables::Expiry => self.movement.time..=f64::MAX,
            Adjustables::EndPrice => 0.0..=f64::MAX,
            Adjustables::EndTime => 0.0..=self.contract.expiry,
            Adjustables::EndVol => 0.0..=f64::MAX,
        }
    }

    /// Update appropriate data given the parameter and a value
    fn set_adjustable(&mut self, var: Adjustables, val: f64) {
        match var {
            Adjustables::Expiry => {self.contract.expiry = val;}
            Adjustables::EndTime => {self.movement.time = val;}
            Adjustables::EndPrice => {self.movement.stock = val;}
            Adjustables::Strike => {self.contract.strike = val;}
            Adjustables::EndVol => {self.end_env.vol = val;}
        }
    }

    /// Retrieves appropriate data given the parameter and a value
    fn get_adjustable(&self, var: Adjustables) -> f64 {
        match var {
            Adjustables::Strike => self.contract.strike,
            Adjustables::Expiry => self.contract.expiry,
            Adjustables::EndPrice => self.movement.stock,
            Adjustables::EndTime => self.movement.time,
            Adjustables::EndVol => self.end_env.vol,
        }
    }

    fn create_chart(&self, y_axis: PayoffYAxis, x_axis: Adjustables) -> PayoffChart {
        let mut chart: PayoffChart;
        if y_axis == PayoffYAxis::Nominal {
            chart = PayoffChart::new_nominal_chart(format!("{} for different {}", y_axis, x_axis), format!("{}", x_axis));
            chart.set_benchmark_height(self.answers.2);
            chart.set_yrange(0.0..=self.answers.3*1.1);
        } else {
            chart = PayoffChart::new_roi_chart(format!("{} for different {}", y_axis, x_axis), format!("{}", x_axis));
            chart.set_yrange(0.0..=self.answers.4*1.1);
        }
        chart.set_xrange(self.ranges[x_axis as usize].clone());
        return chart;
    }

    /// Configures a variable slider within the sliderlist at a given index
    fn configure_slider(&mut self, i: usize) {
        let adj;
        // Check validity of i
        if let Some(&(_adj, _)) = self.sliders.data.get(i) {
            adj = _adj
        } else {
            return;
        }
        let val = self.get_adjustable(adj);
        let range = self.ranges[adj as usize].clone();
        // We know i is valid at this point
        let slider = &mut self.sliders.data[i].1;
        slider.set_allowed_range(range.clone())
            .set_slider_range(range);
        slider.set_value(val);
    }

    /// Configures a payoff chart within the chartlist at a given index
    fn configure_chart(&mut self, i: usize) {
        let (y_axis, x_axis);
        // Check validity of i
        if let Some(((_y_axis, _x_axis), _)) = self.charts.data.get(i) {
            (y_axis, x_axis) = (*_y_axis, *_x_axis);
        } else {
            return;
        }
        let x_range = self.ranges[x_axis as usize].clone();
        let x_val = self.get_adjustable(x_axis);
        let func;
        // If using call options
        if self.answers.0 {
            func = self.get_parameterisation::<Call>(y_axis, x_axis);
        } else { // Elsewise using put option
            func = self.get_parameterisation::<Put>(y_axis, x_axis);
        }
        let (_, chart) = &mut self.charts.data[i];
        chart.set_func(func)
            .set_xrange(x_range)
            .set_x_vert(x_val);

        // Update entry price benchmark
        let mut entry = 1.0;
        if y_axis == PayoffYAxis::Nominal {
            if self.answers.0 {
                entry = Call::bsm_price(&self.start_env, &self.contract);
            } else {
                entry = Put::bsm_price(&self.start_env, &self.contract);
            }
        }
        chart.set_benchmark_height(entry);
    }

    /// Generates a single variable function that encapsulate a blackscholes calculation with 1 variable free. These
    /// should be given to the payoff graphs to be plotted.
    fn get_parameterisation<T: BlackScholesROI>(&self, out: PayoffYAxis, var: Adjustables) -> Box<dyn Fn(f64) -> f64> {
        // Clone appropriate data
        let func0 = {
            let start_env = self.start_env.clone();
            let end_env = self.end_env.clone();
            let contract = self.contract.clone();
            let predict = self.movement.clone();
            Box::new(move |x| (x, start_env.clone(), end_env.clone(), contract.clone(), predict.clone()))
        };

        // Establish data manipulation
        let func1: Box<dyn Fn((f64, Environment, Environment, Contract, Movement)) -> (Environment, Environment, Contract, Movement)>;
        match var {
            Adjustables::Strike => {
                func1 = Box::new(|(x, start_env, end_env, contract, predict)| {
                    let mut new_contract = contract.clone();
                    new_contract.strike = x;
                    (start_env, end_env, new_contract, predict)
                });
            }
            Adjustables::Expiry => {
                func1 = Box::new(|(x, start_env, end_env, contract, predict)| {
                    let mut new_contract = contract.clone();
                    new_contract.expiry = x;
                    (start_env, end_env, new_contract, predict)
                });
            }
            Adjustables::EndPrice => {
                func1 = Box::new(|(x, start_env, end_env, contract, predict)| {
                    let mut new_predict = predict.clone();
                    new_predict.stock = x;
                    (start_env, end_env, contract, new_predict)
                });
            }
            Adjustables::EndTime => {
                func1 = Box::new(|(x, start_env, end_env, contract, predict)| {
                    let mut new_predict = predict.clone();
                    new_predict.time = x;
                    (start_env, end_env, contract, new_predict)
                });
            }
            Adjustables::EndVol => {
                func1 = Box::new(|(x, start_env, end_env, contract, predict)| {
                    let mut new_end_env = end_env.clone();
                    new_end_env.vol = x;
                    (start_env, new_end_env, contract, predict)
                });
            }
        }

        // Establish whether to call ROI or nominal calculation
        let func2: Box<dyn Fn((Environment, Environment, Contract, Movement)) -> f64>;
        match out {
            PayoffYAxis::ROI => {
                func2 = Box::new(|(start_env, end_env, contract, predict)| T::roi(&start_env, &end_env, &contract, &predict));
            }
            PayoffYAxis::Nominal => {
                func2 = Box::new(|(_start_env, mut end_env, contract, predict)| {
                    let end_contract;
                    (end_env, end_contract) = predict.apply(end_env, contract);
                    T::bsm_price(&end_env, &end_contract)
                })
            }
        }

        return Box::new(move |x| func2(func1(func0(x))));
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Calculate => {
                // Validate and extract inputs
                if let Some((env, pred)) = self.extract_env_and_pred() {
                    self.start_env = env.clone();
                    self.end_env = env;
                    self.movement = pred;
                } else {
                    // self.answers_str = [
                    //     String::from("Input invalid"),
                    //     String::new(),
                    //     String::new(),
                    //     String::new(),
                    //     String::new(),
                    //     String::new(),
                    // ];
                    return Task::none();
                }

                
                // Predicting stock to go up then we should use a call option
                if self.movement.stock >= self.start_env.stock {
                    // Find best contract given starting environment and predicted price movement
                    self.contract = Call::find_best_contract(&self.start_env, &self.start_env, &self.movement);
                    let end_contract: Contract;
                    (self.end_env, end_contract) = self.movement.apply(self.start_env.clone(), self.contract.clone());
                    let buy_price = Call::bsm_price(&self.start_env, &self.contract);
                    let sell_price = Call::bsm_price(&self.end_env, &end_contract);
                    let roi = Call::roi(&self.start_env, &self.start_env, &self.contract, &self.movement);

                    self.answers = (
                        true,
                        self.contract.clone(),
                        buy_price,
                        sell_price,
                        roi,
                    );

                } else { // Elsewise we use a put option
                    // Find best contract given starting environment and predicted price movement
                    self.contract = Put::find_best_contract(&self.start_env, &self.start_env, &self.movement);
                    let end_contract: Contract;
                    (self.end_env, end_contract) = self.movement.apply(self.start_env.clone(), self.contract.clone());
                    let buy_price = Call::bsm_price(&self.start_env, &self.contract);
                    let sell_price = Call::bsm_price(&self.end_env, &end_contract);
                    let roi = Put::roi(&self.start_env, &self.start_env, &self.contract, &self.movement);

                    self.answers = (
                        false,
                        self.contract.clone(),
                        buy_price,
                        sell_price,
                        roi,
                    );
                }
                // Configure ranges
                for &adj in Adjustables::everything().iter() {
                    self.ranges[adj as usize] = self.create_default_range(adj);
                }

                // Add the strike sliders if nothing is present
                if self.sliders.data.is_empty() {
                    let mut slider = CustomSlider::default().set_precision(MAX_DP);
                    slider.set_title(format!("{}", Adjustables::Strike));
                    self.sliders.unique_push(Adjustables::Strike, slider);
                }
                // Configure slider values and ranges
                for i in 0..self.sliders.data.len() {
                    self.configure_slider(i);
                }
                return Task::none();
            }
            Message::NumberInputMessage(i, number_msg) => {
                self.param[i].update(number_msg);
                return Task::none();
            }
            Message::Sliders(list_message) => {
                self.sliders.update(list_message.clone());

                // Update appropriate value from parameter slider
                if let DeletableListMessage::Item(i, _) = list_message {
                    let var = self.sliders.data[i].0.clone();
                    let val = self.sliders.data[i].1.get_value();
                    self.set_adjustable(var, val);
                } else {
                    return Task::none();
                }

                // Update valid ranges the sliders can take up
                for i in 0..self.sliders.data.len() {
                    let var = self.sliders.data[i].0.clone();
                    let range = self.create_valid_range(var.clone());
                    self.sliders.data[i].1.set_allowed_range(range);
                    self.ranges[var as usize] = self.sliders.data[i].1.get_slider_range();
                }

                for i in 0..self.charts.data.len() {
                    self.configure_chart(i);
                }
                return Task::none();
            }
            Message::SliderSelect(variable) => {
                self.slider_add_select = Some(variable);
                return Task::none();
            }
            Message::SliderAdd => {
                if let Some(variable) = self.slider_add_select {
                    let mut slider = CustomSlider::default().set_precision(MAX_DP);
                    slider.set_title(format!("{}", variable))
                        .set_allowed_range(0.0..=f64::MAX);
                    self.sliders.unique_push(variable, slider);
                    self.configure_slider(self.sliders.data.len()-1);
                }
                return Task::none();
            }
            Message::ChartXSelect(variable) => {
                self.chart_x_select = Some(variable);
                return Task::none();
            }
            Message::ChartYSelect(y) => {
                self.chart_y_select = Some(y);
                return Task::none();
            }
            Message::ChartAdd => {
                if let (Some(y_axis), Some(x_axis)) = (self.chart_y_select, self.chart_x_select) {
                    let mut chart = self.create_chart(y_axis, x_axis);
                    let func;
                    // Using call options
                    if self.answers.0 == true {
                        func = self.get_parameterisation::<Call>(y_axis, x_axis)
                    } else { // Else using put options
                        func = self.get_parameterisation::<Put>(y_axis, x_axis)
                    }
                    chart.set_func(func);
                    self.charts.unique_push((y_axis, x_axis), chart);
                }
                return Task::none();
            }
            Message::Charts(list_msg) => {
                self.charts.update(list_msg);
                return Task::none();
            }
            Message::TabPressed => {
                return operation::focus_next();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        row![
            scrollable(column![
                tooltip(
                    text("Environment").size(30),
                    container("Details about the stock in the current moment")
                        .padding(5)
                        .style(container::rounded_box),
                    tooltip::Position::FollowCursor
                ),
                text!("Stock price"),
                self.param[0].view().map(|number_msg| Message::NumberInputMessage(0, number_msg)),
                text!("Volatility"),
                self.param[1].view().map(|number_msg| Message::NumberInputMessage(1, number_msg)),
                text!("Risk free rate"),
                self.param[2].view().map(|number_msg| Message::NumberInputMessage(2, number_msg)),
                text!("Dividend yield"),
                self.param[3].view().map(|number_msg| Message::NumberInputMessage(3, number_msg)),

                rule::horizontal(2),

                tooltip(
                    text("Prediction").size(30),
                    container("Prediction on what price the stock will reach and when")
                        .padding(5)
                        .style(container::rounded_box),
                    tooltip::Position::FollowCursor
                ),
                text!("Prediction stock price"),
                self.param[4].view().map(|number_msg| Message::NumberInputMessage(4, number_msg)),
                text!("Prediction end duration"),
                self.param[5].view().map(|number_msg| Message::NumberInputMessage(5, number_msg)),
                button("Calculate").on_press(Message::Calculate),

                rule::horizontal(2),

                tooltip(
                    Column::with_children(
                        self.answer_text_block().into_iter().map(|s| text(s).size(20).into())
                    ),
                    container("The option contract to buy immediately and to sell \nat the prediction end duration that maximises ROI. \nAssumes that: \n - Environment variables (except stock price)\n   stay constant \n - Prediction becomes perfectly true")
                        .padding(5)
                        .style(container::rounded_box),
                    tooltip::Position::FollowCursor
                ),

                rule::horizontal(2),

                self.sliders.view(|x| x.spacing(5)).map(|msg| Message::Sliders(msg)),
                row![
                    pick_list(Adjustables::everything(), self.slider_add_select, Message::SliderSelect)
                        .placeholder("Choose Variable"),
                    button("Add Slider").on_press(Message::SliderAdd),
                ]
            ].padding(20)
            .spacing(5)
            .width(350)
            .align_x(Left)),

            rule::vertical(2),

            responsive( |area| {
                scrollable(
                    column![
                        container(self.charts.view(|x| x).map(|msg| Message::Charts(msg)))
                        .height((0.5 * area.height * self.charts.data.len() as f32) - 80.0),
                        container(row![
                            pick_list(PayoffYAxis::everything(), self.chart_y_select, Message::ChartYSelect)
                                .placeholder("Choose Y-axis Content"),
                            pick_list(Adjustables::everything(), self.chart_x_select, Message::ChartXSelect)
                                .placeholder("Choose X-axis Content"),
                            button("Add Chart").on_press(Message::ChartAdd),
                        ]).width(Length::Fill).align_x(Center)
                    ]
                    .padding(20)
                    .spacing(5)
                    .align_x(Center)
                ).into()
            })
        ].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::keyboard;

        keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key),
                modifiers,
                ..
            } => match (key, modifiers) {
                (keyboard::key::Named::Tab, _) => Some(Message::TabPressed),
                _ => None,
            }
            _ => None,
        })
    }
}


