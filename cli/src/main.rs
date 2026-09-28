mod parameters;

use std::{
    iter::{repeat_n, zip},
    io,
    sync::mpsc::{channel, Sender},
    time::{Duration, Instant},
    thread,
    thread::{JoinHandle},
    error,
    ops::{Bound},
    collections::HashMap,
    fmt::Display
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Flex},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device,
    SampleFormat,
    BufferSize,
    Stream,
    SupportedStreamConfig,
    StreamConfig,
};
use ratatui::widgets::{BorderType, Wrap};
use analysis::{
    OutputMessage,
    AudioMessage,
    start_process_thread,
    get_bins,
    detection::{
        ErrorTypes,
        FrameContext,
        sub_harmonic_summing,
        peak_picking,
        two_way_mismatch,
    }
};
use phf::{
    Map as phfMap,
    phf_map
};

use parameters::{
    Parameter,
    BoundedParameter,
    LinearParam,
    ExponentialParam,
    CyclicParam,
    BoolParam,
    BoundedParam,
};

/*
{Underlined} Name                   |
Arrow Up / Down OR Scroll Wheel     |
Range: (0 - 1]                      |
Suggested: [0.6 - 0.8]              |
                                    |
{Yellow} ABOVE SUGGESTED RANGE      |
OR {Yellow} BELOW SUGGESTED RANGE   |
(OR) {Red} AT MINIMUM               |
(OR) {Red} AT MAXIMUM               |
(OR) {GREEN} WITHIN SUGGESTED RANGE |
*/


static LETTERS: &[&str] = &["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
static LETTERS_DISP: &[&str] = &{[
r"
   ___________
  /           |
 /     _______|
|     /
|    |
|    |
|    |
|    |
|     \_______
 \            |
  \___________|
",
r"
   ___________   __   __
  /           | |  | |  |
 /     _______|_|  |_|  |__
|     /      |             |
|    |       |__    _    __|
|    |          |  | |  |
|    |        __|  |_|  |__
|    |       |             |
|     \______|__    _    __|
 \            | |  | |  |
  \___________| |__| |__|
",
r"
 ___________
|           \
|     ___    \
|    |   \    |
|    |   |    |
|    |   |    |
|    |   |    |
|    |   |    |
|    |__/     |
|            /
|___________/
",
r"

 ___________      __   __
|           \    |  | |  |
|     ___    \ __|  |_|  |__
|    |   \    |             |
|    |   |    |__    _    __|
|    |   |    |  |  | |  |
|    |   |    |__|  |_|  |__
|    |   |    |             |
|    |__/     |__    _    __|
|            /   |  | |  |
|___________/    |__| |__|
",
r"
 _____________
|             |
|      _______|
|     |
|     |_______
|             |
|      _______|
|     |
|     |_______
|             |
|_____________|
",
r"
 _____________
|             |
|      _______|
|     |
|     |_______
|             |
|      _______|
|     |
|     |
|     |
|_____|
",
r"
 _____________   __   __
|             | |  | |  |
|      _______|_|  |_|  |__
|     |      |             |
|     |____  |__    _    __|
|          |    |  | |  |
|      ____|  __|  |_|  |__
|     |      |             |
|     |      |__    _    __|
|     |         |  | |  |
|_____|         |__| |__|
",
r"
   ___________
  /           |
 /     _______|
|     /
|    |
|    |  .-----.
|    |  |_    |
|    |    |   |
|     \__/    |
 \           /
  \_________/
",
r"
   ___________    __   __
  /           |  |  | |  |
 /     _______|__|  |_|  |__
|     /       |             |
|    |        |__    _    __|
|    |  .-----.  |  | |  |
|    |  |_    |__|  |_|  |__
|    |    |   |             |
|     \__/    |__    _    __|
 \           /   |  | |  |
  \_________/    |__| |__|
",
r"
    _______
   /       \
  /   ___   \
 /   /   \   \
|   |     |   |
|   |_____|   |
|             |
|    _____    |
|   |     |   |
|   |     |   |
|___|     |___|
",
r"
    _______       __   __
   /       \     |  | |  |
  /   ___   \  __|  |_|  |__
 /   /   \   \|             |
|   |     |   |__    _    __|
|   |_____|   |  |  | |  |
|             |__|  |_|  |__
|    _____    |             |
|   |     |   |__    _    __|
|   |     |   |  |  | |  |
|___|     |___|  |__| |__|
",
r"
 ___________
|           \
|    ____    \
|   |    \    |
|   |____/    |
|            /
|    ____    \
|   |    \    |
|   |____/    |
|            /
|___________/
"]};

static NUMBERS: &[&str] = &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
static NUMBERS_DISP: &[&str] = &{[
r"
   _________
  /         \
 /    ___    \
|    /   \    |
|   |     |   |
|   |     |   |
|   |     |   |
|   |     |   |
|    \___/    |
 \           /
  \_________/
",
r"
   ____
 /     |
/__    |
   |   |
   |   |
   |   |
   |   |
   |   |
   |   |
   |   |
   |___|
",
r"
 ________
|        \
|_____    \
      )    |
     /    /
    /    /
   /    /
  /    /
 /    /____
|          |
|__________|
",
r"
 _________
|         \
|______    \
       \    |
 ______/    |
|          /
|______    \
       \    |
 ______/    |
|          /
|_________/
",
r"
 ___   ___
|   | |   |
|   | |   |
|   | |   |
|   |_|   |__
|            |
|_____     __|
      |   |
      |   |
      |   |
      |___|
",
r"
 __________
|          |
|    ______|
|   |
|   |____
|        \
|_____    \
      \    |
 _____/    |
|         /
|________/
",
r"
     _____
    /    /
   /    /
  /    /
 /    /_______
|             \
|     _____    \
|    |     \    |
|     \____/    |
 \             /
  \___________/
",
r"
 ______________
|              |
|_________     |
         /    /
        /    /
       /    /
      /    /
     /    /
    /    /
   /    /
  /____/
",
r"
   ________
  /        \
 /    __    \
|    /  \    |
|    \__/    |
 \          /
 /    __    \
|    /  \    |
|    \__/    |
 \          /
  \________/
",
r"
   ________
  /        \
 /    __    \
|    /  \    |
|    \__/    |
 \          /
  \____    /
      /   /
     /   /
    /   /
   /___/
"]};

static NO_NOTE_DISP: &str = {r"
 ____     ____   _________   ____     ____ _____________
|    \   |    | /         \ |    \   |    |             |
|     \  |    |/    ___    \|     \  |    |      _______|
|      \ |    |    /   \    |      \ |    |     |
|       \|    |   |     |   |       \|    |     |_______
|        `    |   |     |   |        `    |             |
|    .        |   |     |   |    .        |      _______|
|    |\       |   |     |   |    |\       |     |
|    | \      |    \___/    |    | \      |     |_______
|    |  \     |\           /|    |  \     |             |
|____|   \____| \_________/ |____|   \____|_____________|
"};

static ERROR_DISP: &str = {r"
 _____________ ___________   ___________     _________   ___________
|             |           \ |           \   /         \ |           \
|      _______|    ____    \|    ____    \ /    ___    \|    ____    \
|     |       |   |    \    |   |    \    |    /   \    |   |    \    |
|     |_______|   |     |   |   |     |   |   |     |   |   |     |   |
|             |   |____/   /|   |____/   /|   |     |   |   |____/   /
|      _______|           / |           / |   |     |   |           /
|     |       |    ____   \ |    ____   \ |   |     |   |    ____   \
|     |_______|   |    \   \|   |    \   \|    \___/    |   |    \   \
|             |   |     |   |   |     |   |\           /|   |     |   |
|_____________|___|     |___|___|     |___| \_________/ |___|     |___|
"};

static FREQUENCIES: &[f32] = &{[
    16.35160,
    17.32391,
    18.35405,
    19.44544,
    20.60172,
    21.82676,
    23.12465,
    24.49971,
    25.95654,
    27.50000,
    29.13524,
    30.86771,
]};

static DETECT_METHODS: &[&str] = &{[
    "Sub-Harmonic Summing (Default)",
    "Peak Picking",
    "Two-Way Mismatch",
]};
/*
    "Spectral Autocorrelation",
    "Comb Distance",
    "Template Matching",
*/

static LSB_UI: &[&str] = &{[
    "min_freq",
    "max_freq",
    "vol_filter",
    "detect_mode",
    "vol_steepness",
]};
static PITCH_DISPLAY_UI: &[&str] = &{[
    "pitch",
]};
static VOL_DISPLAY_UI: &[&str] = &{[
    "vol"
]};
static RSB_SHS_UI: &[&str] = &{[
    "num_harmonics",
    "harmonic_decay",
    "comp_turn_point",
    "favor_low",
    "peak_threshold",
]};
static RSB_TWM_UI: &[&str] = &{[
    "predicted_harms",
    "measured_peaks",
    "freq_penalty",
    "amp_weight",
    "freq_weight",
    "error_ratio",
]};

static UI_ID_TO_NAMES: phfMap<&'static str, &'static str> = phf_map! {
    "min_freq" => "Minimum Frequency",
    "max_freq" => "Maximum Frequency",
    "vol_filter" => "Volume Filter",
    "detect_mode" => "Detection Mode",
    "vol_steepness" => "Volume Steepness",
    "pitch" => "Pitch Display",
    "vol" => "Volume Display", // shortened to "Volume" in widget
    "num_harmonics" => "Number of Harmonics",
    "harmonic_decay" => "Harmonic Decay",
    "comp_turn_point" => "Compression Turning Point",
    "favor_low" => "Favor Low Frequencies?",
    "peak_threshold" => "Peak Threshold",
    "predicted_harms" => "Number of Predicted Harmonics",
    "measured_peaks" => "Number of Measured Peaks",
    "freq_penalty" => "Frequency Penalty",
    "amp_weight" => "Amplitude Weight",
    "freq_weight" => "Frequency Weight",
    "error_ratio" => "Error Ratio",
};

static UI_IDS_W_SUGGESTED: &[&str] = &{[
    LSB_UI[2],
    LSB_UI[4],
    RSB_SHS_UI[0],
    RSB_SHS_UI[1],
    RSB_SHS_UI[2],
    RSB_SHS_UI[3],
    RSB_SHS_UI[4],
    RSB_TWM_UI[0],
    RSB_TWM_UI[1],
    RSB_TWM_UI[2],
    RSB_TWM_UI[3],
    RSB_TWM_UI[4],
    RSB_TWM_UI[5],
]};

static UI_IDS_W_DIRECTION: &[&str] = &{[
    LSB_UI[0],
    LSB_UI[1],
    LSB_UI[2],
    LSB_UI[4],
    RSB_SHS_UI[0],
    RSB_SHS_UI[1],
    RSB_SHS_UI[2],
    RSB_SHS_UI[4],
    RSB_TWM_UI[0],
    RSB_TWM_UI[1],
    RSB_TWM_UI[2],
    RSB_TWM_UI[3],
    RSB_TWM_UI[4],
    RSB_TWM_UI[5],
]};

static COLOR_BASES: phfMap<&'static str, (u8, u8, u8)> = phf_map! {
    "blue" => (57, 147, 212),
    "green" => (35, 82, 21),
    "darkblue" => (21, 68, 101),
    "lightgreen" => (80, 186, 48),
    "red" => (224, 77, 77),
    "yellow" => (231, 166, 54),
    "grey" => (143, 143, 143),
};


fn main() -> Result<(), Box<dyn error::Error>> {
    let mut colors: HashMap<&str, Color> = HashMap::new();
    for (&key, &value) in &COLOR_BASES {
        colors.insert(key, get_color(value));
    }
    let colors = colors;

    let no_note_ref = normalize_disp_str(NO_NOTE_DISP);

    let (in_post, in_mail) = channel::<OutputMessage>();

    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;


    let min_pos_freq: f32 = FREQUENCIES[0];
    let max_pos_freq: f32 = FREQUENCIES[11] * 2.0f32.powf(9.0);

    // Frequency Parameters
    let mut min_freq = {
        ExponentialParam::new(
            FREQUENCIES[0] * 2.0f32.powf(2.0),
            2.0f32.powf(1.0 / 12.0)
        )
    };
    let mut max_freq = {
        ExponentialParam::new(
            FREQUENCIES[11] * 2.0f32.powf(6.0),
            2.0f32.powf(1.0 / 12.0)
        )
    };

    let mut min_freq_bound: f32 = min_freq.shift_up();
    let mut max_freq_bound: f32 = max_freq.shift_down();
    let mut bins = get_bins(min_freq.get(), max_freq.get(), 12);

    let (_device, audio_stream, out_post, handle) = setup_audio_callback(min_freq.get(), max_freq.get(), 12, in_post, BufferSize::Fixed(512))?;

    // Volume Parameters
    let mut vol_filter = { // 0.003 - 0.010
        BoundedParam::new(
            LinearParam::new(
                0.005f32,
                0.001),
            Bound::Included(0.0),
            Bound::Included(1000.0)
        )
    };
    let mut vol_steepness = { // 2 - 4
        BoundedParam::new(
            LinearParam::new(
                2,
                1),
            Bound::Included(2),
            Bound::Included(40),
        )
    };

    // Detection Parameters
    let mut detect_mode = {
        CyclicParam::new(
            0,
            1,
            DETECT_METHODS.len(),
            0
        ).map_err(|_| "invalid bounds for detect mode")?
    };
    let mut shs_num_harmonics = { // 3 - 8
        BoundedParam::new(
            LinearParam::new(4u8, 1),
            Bound::Included(1),
            Bound::Included(20)
        )
    };
    let mut shs_harmonic_decay = { // 0.7 - 0.9
        BoundedParam::new(
            LinearParam::new(0.85f32, 0.05),
            Bound::Included(0.0),
            Bound::Included(1.0)
        )
    };
    let mut shs_comp_turn_point = { // 10 - 1000
        BoundedParam::new(
            ExponentialParam::new(100.0f32, 10.0f32.powf(1.0f32 / 3.0)),
            Bound::Included(1.0),
            Bound::Included(100000000.0),
        )
    };
    let mut shs_favor_low = BoolParam::new(true); // true
    let mut shs_peak_threshold = { // 0.8 - 0.95
        BoundedParam::new(
            LinearParam::new(0.9f32, 0.02),
            Bound::Included(0.0),
            Bound::Included(1.0)
        )
    };

    let mut twm_predicted_harms = { // 8 - 15; changes max detected frequency
        BoundedParam::new(
            LinearParam::new(10usize, 1),
            Bound::Included(3),
            Bound::Included(20),
        )
    };
    let mut twm_measured_peaks = { // 8 - 15
        BoundedParam::new(
            LinearParam::new(10usize, 1),
            Bound::Included(2),
            Bound::Included(20),
        )
    };
    let mut twm_freq_penalty = { // 0.25 - 0.75
        BoundedParam::new(
            LinearParam::new(0.5f32, 0.05),
            Bound::Included(0.0),
            Bound::Included(2.0),
        )
    };
    let mut twm_amp_weight = { // 0.05 - 0.20
        BoundedParam::new(
            ExponentialParam::new(0.1f32, 2.0),
            Bound::Included(0.0625),
            Bound::Included(20.0),
        )
    };
    let mut twm_freq_weight = { // 0.2 - 0.8
        BoundedParam::new(
            ExponentialParam::new(0.4f32, 2.0),
            Bound::Included(0.0625),
            Bound::Included(20.0),
        )
    };
    let mut twm_error_ratio = { // 0.2 - 0.6
        BoundedParam::new(
            LinearParam::new(0.5f32, 0.05),
            Bound::Included(0.0),
            Bound::Included(1.0)
        )
    };

    let mut current_note: String = normalize_disp_str(NO_NOTE_DISP);
    let mut last_note = Instant::now();
    let mut last_vol: f32 = 0.0;

    let mut selected = "None";
    let mut cursor_x: Option<usize> = None;
    let mut cursor_y = 0usize;
    let mut last_cursor_x = Some(1usize);

    audio_stream.play()?;

    let mut wait = true;

    'main: loop {
        while let Ok(msg) = in_mail.try_recv() {
            match msg {
                OutputMessage::NewFreqFrame(frame_data) => {
                    last_vol = *frame_data.iter()
                        .max_by(|a, b| a.total_cmp(b)).unwrap();

                    let mut max_ind: Result<usize, usize> = Err(0);

                    if last_vol > vol_filter.get() {
                        let frame_context = FrameContext {
                            frame_data: &frame_data,
                            bins: &bins,
                            bins_per_octave: 12,
                        };
                        if detect_mode == 0 {
                            match sub_harmonic_summing(frame_context,
                                                       shs_num_harmonics.get(),
                                                       shs_harmonic_decay.get(),
                                                       shs_comp_turn_point.get(),
                                                       shs_favor_low.get(),
                                                       shs_peak_threshold.get()) {
                                Ok(max_index) => max_ind = Ok(max_index),
                                Err(ErrorTypes::BreakError(desc)) => Err(desc)?,
                                Err(ErrorTypes::StandardError) => max_ind = Err(0),
                            }
                        } else if detect_mode == 1 {
                            match peak_picking(frame_context) {
                                Ok(max_index) => max_ind = Ok(max_index),
                                Err(ErrorTypes::BreakError(desc)) => Err(desc)?,
                                Err(ErrorTypes::StandardError) => max_ind = Err(0),
                            }
                        } else if detect_mode == 2 {
                            match two_way_mismatch(frame_context,
                                                   twm_predicted_harms.get(),
                                                   twm_measured_peaks.get(),
                                                   twm_freq_penalty.get(),
                                                   twm_amp_weight.get(),
                                                   twm_freq_weight.get(),
                                                   twm_error_ratio.get()) {
                                Ok(max_index) => max_ind = Ok(max_index),
                                Err(ErrorTypes::BreakError(desc)) => Err(desc)?,
                                Err(ErrorTypes::StandardError) => max_ind = Err(0),
                            }
                        }

                        match max_ind {
                            Ok(max_index) => {
                                match freq_to_inds(bins[max_index]) {
                                    Ok((let_ind, oct_ind)) => {
                                        current_note = horizontal_concat(&normalize_disp_str(LETTERS_DISP[let_ind]), &normalize_disp_str(NUMBERS_DISP[oct_ind]));
                                    },
                                    Err((_, desc)) => Err(desc)?
                                }
                            },
                            Err(0) => {
                                current_note = normalize_disp_str(NO_NOTE_DISP)
                            },
                            Err(_) => {
                                current_note = normalize_disp_str(ERROR_DISP)
                            },
                        };
                        last_note = Instant::now();
                    }
                }
                OutputMessage::ConfirmMinFreq(new_min, accepted, _desc) if accepted => {
                    min_freq.set(new_min);
                    min_freq_bound = min_freq.shift_up();
                    bins = get_bins(min_freq.get(), max_freq.get(), 12);
                }
                OutputMessage::ConfirmMaxFreq(new_max, accepted, _desc) if accepted => {
                    max_freq.set(new_max);
                    max_freq_bound = max_freq.shift_down();
                    bins = get_bins(min_freq.get(), max_freq.get(), 12);
                }
                OutputMessage::ConfirmBounds(new_min, new_max, accepted, _desc) if accepted => {
                    min_freq.set(new_min);
                    max_freq.set(new_max);
                    min_freq_bound = min_freq.shift_up();
                    max_freq_bound = max_freq.shift_down();
                    bins = get_bins(min_freq.get(), max_freq.get(), 12);

                }
                _ => {}
            }
        }
        if last_note.elapsed() > Duration::from_secs_f32(0.02) && current_note != no_note_ref {
            current_note = normalize_disp_str(NO_NOTE_DISP);
        }

        terminal.draw(|frame| {
            let screen_area = frame.area();

            if cursor_x.is_some() {
                cursor_x = Some(cursor_x.unwrap().clamp(0, 3));
                if detect_mode == 1 {
                    cursor_x = Some(cursor_x.unwrap().clamp(0, 2));
                }
            }

            let main_app_height = 25;
            let tooltips_height = 12;

            let left_sidebar_width = 35;
            let pitch_display_width = 81;
            let vol_display_width = 10;
            let right_sidebar_width = 35;
            let tooltips_width = left_sidebar_width + 2 + pitch_display_width + 2 + vol_display_width + 2 + right_sidebar_width;

            let [vertical_center] = {
                Layout::vertical([Constraint::Length(main_app_height + tooltips_height)])
                    .flex(Flex::Center)
                    .areas(screen_area)
            };

            let [main_app_area, _vspacer, bottom_app_area] = {
                Layout::vertical([
                    Constraint::Length(main_app_height),
                    Constraint::Length(1),
                    Constraint::Length(tooltips_height),
                ]).flex(Flex::Center)
                    .areas(vertical_center)
            };

            let [left_sidebar_area, _spacer1, pitch_display_area, _spacer2, vol_display_area, _spacer3, right_sidebar_area] = {
                Layout::horizontal([
                    Constraint::Length(left_sidebar_width),
                    Constraint::Length(2),
                    Constraint::Length(pitch_display_width),
                    Constraint::Length(2),
                    Constraint::Length(vol_display_width),
                    Constraint::Length(2),
                    Constraint::Length(right_sidebar_width)
                ]).flex(Flex::Center)
                    .areas(main_app_area)
            };

            // Render Left Sidebar
            {
                if cursor_x == Some(0) {
                    cursor_y = cursor_y.clamp(0, LSB_UI.len() - 1);
                    selected = LSB_UI[cursor_y];
                }

                let lsb_min_freq_widget = {
                    let (min_let_ind, min_oct_ind) = freq_to_inds(min_freq.get()).ok().unwrap();
                    Paragraph::new(format!("{} / {}{}", min_freq.get(), LETTERS[min_let_ind], NUMBERS[min_oct_ind]))
                        .style(Style::default()
                            .fg(get_text_fg(selected, LSB_UI[0], &colors))
                            .bg(get_text_bg(selected, LSB_UI[0], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, LSB_UI[0], &colors))
                                .bg(get_border_bg(selected, LSB_UI[0], &colors))))
                };
                let lsb_max_freq_widget = {
                    let (max_let_ind, max_oct_ind) = freq_to_inds(max_freq.get()).ok().unwrap();
                    Paragraph::new(format!("{} / {}{}", max_freq.get(), LETTERS[max_let_ind], NUMBERS[max_oct_ind]))
                        .style(Style::default()
                            .fg(get_text_fg(selected, LSB_UI[1], &colors))
                            .bg(get_text_bg(selected, LSB_UI[1], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, LSB_UI[1], &colors))
                                .bg(get_border_bg(selected, LSB_UI[1], &colors))))
                };
                let lsb_vol_filter_widget = {
                    Paragraph::new(format!("{}", vol_filter.get()))
                        .style(Style::default()
                            .fg(get_text_fg(selected, LSB_UI[2], &colors))
                            .bg(get_text_bg(selected, LSB_UI[2], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, LSB_UI[2], &colors))
                                .bg(get_border_bg(selected, LSB_UI[2], &colors))))
                };
                let lsb_detect_mode_widget = {
                    Paragraph::new(format!("{} - {}", detect_mode.get(), DETECT_METHODS[detect_mode.get()]))
                        .style(Style::default()
                            .fg(get_text_fg(selected, LSB_UI[3], &colors))
                            .bg(get_text_bg(selected, LSB_UI[3], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, LSB_UI[3], &colors))
                                .bg(get_border_bg(selected, LSB_UI[3], &colors))))
                };
                let lsb_vol_steepness_widget = {
                    Paragraph::new(format!("{}", vol_steepness.get()))
                        .style(Style::default()
                            .fg(get_text_fg(selected, LSB_UI[4], &colors))
                            .bg(get_text_bg(selected, LSB_UI[4], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, LSB_UI[4], &colors))
                                .bg(get_border_bg(selected, LSB_UI[4], &colors))))
                };

                let [lsb_min_freq_area, _lsb_spacer1, lsb_max_freq_area, _lsb_spacer2, lsb_str_filer_area, _lsb_spacer3, lsb_detect_mode_area, _lsb_spacer4, lsb_vol_steepness_area] = {
                    Layout::vertical([
                        Constraint::Length((lsb_min_freq_widget.line_count(left_sidebar_width - 2) + 2) as u16),
                        Constraint::Length(1),
                        Constraint::Length((lsb_max_freq_widget.line_count(left_sidebar_width - 2) + 2) as u16),
                        Constraint::Length(1),
                        Constraint::Length((lsb_vol_filter_widget.line_count(left_sidebar_width - 2) + 2) as u16),
                        Constraint::Length(1),
                        Constraint::Length((lsb_detect_mode_widget.line_count(left_sidebar_width - 2) + 2) as u16),
                        Constraint::Length(1),
                        Constraint::Length((lsb_vol_steepness_widget.line_count(left_sidebar_width - 2) + 2) as u16),
                    ]).flex(Flex::Start)
                        .areas(left_sidebar_area)
                };

                let lsb_min_freq_border_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, LSB_UI[0], &colors))
                            .bg(get_border_bg(selected, LSB_UI[0], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[LSB_UI[0]]))
                };
                let lsb_max_freq_border_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, LSB_UI[1], &colors))
                            .bg(get_border_bg(selected, LSB_UI[1], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[LSB_UI[1]]))
                };
                let lsb_vol_filter_border_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, LSB_UI[2], &colors))
                            .bg(get_border_bg(selected, LSB_UI[2], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[LSB_UI[2]]))
                };
                let lsb_detect_mode_border_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, LSB_UI[3], &colors))
                            .bg(get_border_bg(selected, LSB_UI[3], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[LSB_UI[3]]))
                };
                let lsb_vol_steepness_border_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, LSB_UI[4], &colors))
                            .bg(get_border_bg(selected, LSB_UI[4], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[LSB_UI[4]]))
                };

                let lsb_min_freq_content_area = lsb_min_freq_border_block.inner(lsb_min_freq_area);
                let lsb_max_freq_content_area = lsb_max_freq_border_block.inner(lsb_max_freq_area);
                let lsb_vol_filter_content_area = lsb_vol_filter_border_block.inner(lsb_str_filer_area);
                let lsb_detect_mode_content_area = lsb_detect_mode_border_block.inner(lsb_detect_mode_area);
                let lsb_vol_steepness_content_area = lsb_vol_steepness_border_block.inner(lsb_vol_steepness_area);

                frame.render_widget(lsb_min_freq_border_block, lsb_min_freq_area);
                frame.render_widget(lsb_max_freq_border_block, lsb_max_freq_area);
                frame.render_widget(lsb_vol_filter_border_block, lsb_str_filer_area);
                frame.render_widget(lsb_detect_mode_border_block, lsb_detect_mode_area);
                frame.render_widget(lsb_vol_steepness_border_block, lsb_vol_steepness_area);

                frame.render_widget(lsb_min_freq_widget, lsb_min_freq_content_area);
                frame.render_widget(lsb_max_freq_widget, lsb_max_freq_content_area);
                frame.render_widget(lsb_vol_filter_widget, lsb_vol_filter_content_area);
                frame.render_widget(lsb_detect_mode_widget, lsb_detect_mode_content_area);
                frame.render_widget(lsb_vol_steepness_widget, lsb_vol_steepness_content_area);
            }

            // Render Pitch Display
            {
                if cursor_x == Some(1) {
                    cursor_y = cursor_y.clamp(0, PITCH_DISPLAY_UI.len() - 1);
                    selected = PITCH_DISPLAY_UI[cursor_y];
                }

                let pitch_display_block1 = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, PITCH_DISPLAY_UI[0], &colors))
                            .bg(get_border_bg(selected, PITCH_DISPLAY_UI[0], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[PITCH_DISPLAY_UI[0]]))
                };
                let pitch_display_block2 = {
                    Block::default()
                        .borders(Borders::LEFT | Borders::RIGHT)
                        .border_type(BorderType::QuadrantOutside)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, PITCH_DISPLAY_UI[0], &colors))
                            .bg(get_border_bg(selected, PITCH_DISPLAY_UI[0], &colors)))
                };

                let pitch_display_block2_area = pitch_display_block1.inner(pitch_display_area);
                let pitch_display_content_area = pitch_display_block2.inner(pitch_display_block2_area);

                let pitch_display_content_height = current_note.lines().count() as u16;
                let pitch_display_content_width = current_note.lines().next().map_or(0, |line| line.chars().count()) as u16;

                let [pitch_display_content_vertical] = {
                    Layout::vertical([Constraint::Length(pitch_display_content_height)])
                        .flex(Flex::Center)
                        .areas(pitch_display_content_area)
                };
                let [pitch_display_content_horizontal] = {
                    Layout::horizontal([Constraint::Length(pitch_display_content_width)])
                        .flex(Flex::Center)
                        .areas(pitch_display_content_vertical)
                };

                let mut pitch_display_content_widget = Paragraph::new(current_note.to_string()).alignment(Alignment::Center);
                if current_note == no_note_ref {
                    pitch_display_content_widget = pitch_display_content_widget.style(Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD));
                } else {
                    pitch_display_content_widget = pitch_display_content_widget.style(Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD));
                }

                frame.render_widget(pitch_display_block1, pitch_display_area);
                frame.render_widget(pitch_display_block2, pitch_display_block2_area);

                frame.render_widget(pitch_display_content_widget, pitch_display_content_horizontal);
            }

            // Render Volume Display
            {
                if cursor_x == Some(2) {
                    cursor_y = cursor_y.clamp(0, VOL_DISPLAY_UI.len() - 1);
                    selected = VOL_DISPLAY_UI[cursor_y];
                }

                let vol_display_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, VOL_DISPLAY_UI[0], &colors))
                            .bg(get_border_bg(selected, VOL_DISPLAY_UI[0], &colors)))
                        .title(" Volume ")
                };
                let vol_display_widget = {
                    Paragraph::new(create_bar(normalize(last_vol, vol_filter.get(), vol_steepness.get()),
                                              (vol_display_width - 4) as usize,
                                              (main_app_height - 2) as usize))
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, VOL_DISPLAY_UI[0], &colors))
                                .bg(get_border_bg(selected, VOL_DISPLAY_UI[0], &colors))))
                };

                let vol_display_content_area = vol_display_block.inner(vol_display_area);

                frame.render_widget(vol_display_block, vol_display_area);
                frame.render_widget(vol_display_widget, vol_display_content_area);
            }

            // Render Right Sidebar
            if detect_mode == 0 {
                if cursor_x == Some(3) {
                    cursor_y = cursor_y.clamp(0, RSB_SHS_UI.len() - 1);
                    selected = RSB_SHS_UI[cursor_y];
                }
                // Sub-Harmonic Summing
                let rsb_num_harmonics_widget = {
                    Paragraph::new(format!("{}", shs_num_harmonics.get()))
                        .style(Style::default()
                            .fg(get_text_fg(selected, RSB_SHS_UI[0], &colors))
                            .bg(get_text_bg(selected, RSB_SHS_UI[0], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, RSB_SHS_UI[0], &colors))
                                .bg(get_border_bg(selected, RSB_SHS_UI[0], &colors))))
                };
                let rsb_harmonic_decay_widget = {
                    Paragraph::new(format!("{}", shs_harmonic_decay.get()))
                        .style(Style::default()
                            .fg(get_text_fg(selected, RSB_SHS_UI[1], &colors))
                            .bg(get_text_bg(selected, RSB_SHS_UI[1], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, RSB_SHS_UI[1], &colors))
                                .bg(get_border_bg(selected, RSB_SHS_UI[1], &colors))))
                };
                let rsb_comp_turn_point_widget = {
                    Paragraph::new(format!("{}", shs_comp_turn_point.get()))
                        .style(Style::default()
                            .fg(get_text_fg(selected, RSB_SHS_UI[2], &colors))
                            .bg(get_text_bg(selected, RSB_SHS_UI[2], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, RSB_SHS_UI[2], &colors))
                                .bg(get_border_bg(selected, RSB_SHS_UI[2], &colors))))
                };
                let rsb_favor_low_widget = {
                    Paragraph::new((if shs_favor_low.get() { "Yes" } else { "No" }).to_string())
                        .style(Style::default()
                            .fg(get_text_fg(selected, RSB_SHS_UI[3], &colors))
                            .bg(get_text_bg(selected, RSB_SHS_UI[3], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, RSB_SHS_UI[3], &colors))
                                .bg(get_border_bg(selected, RSB_SHS_UI[3], &colors))))
                };
                let rsb_peak_threshold_widget = {
                    Paragraph::new(format!("{}", shs_peak_threshold.get()))
                        .style(Style::default()
                            .fg(get_text_fg(selected, RSB_SHS_UI[4], &colors))
                            .bg(get_text_bg(selected, RSB_SHS_UI[4], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, RSB_SHS_UI[4], &colors))
                                .bg(get_border_bg(selected, RSB_SHS_UI[4], &colors))))
                };

                let [rsb_num_harmonics_area, _rsb_spacer1, rsb_harmonic_decay_area, _rsb_spacer2, rsb_comp_turn_point_area, _rsb_spacer3, rsb_favor_low_area, _rsb_spacer4, rsb_peak_threshold_area] = {
                    Layout::vertical([
                        Constraint::Length((rsb_num_harmonics_widget.line_count(right_sidebar_width - 2) + 2) as u16),
                        Constraint::Length(1),
                        Constraint::Length((rsb_harmonic_decay_widget.line_count(right_sidebar_width - 2) + 2) as u16),
                        Constraint::Length(1),
                        Constraint::Length((rsb_comp_turn_point_widget.line_count(right_sidebar_width - 2) + 2) as u16),
                        Constraint::Length(1),
                        Constraint::Length((rsb_favor_low_widget.line_count(right_sidebar_width - 2) + 2) as u16),
                        Constraint::Length(1),
                        Constraint::Length((rsb_peak_threshold_widget.line_count(right_sidebar_width - 2) + 2) as u16),
                    ]).flex(Flex::Start)
                        .areas(right_sidebar_area)
                };

                let rsb_num_harmonics_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, RSB_SHS_UI[0], &colors))
                            .bg(get_border_bg(selected, RSB_SHS_UI[0], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[RSB_SHS_UI[0]]))
                };
                let rsb_harmonic_decay_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, RSB_SHS_UI[1], &colors))
                            .bg(get_border_bg(selected, RSB_SHS_UI[1], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[RSB_SHS_UI[1]]))
                };
                let rsb_comp_turn_point_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, RSB_SHS_UI[2], &colors))
                            .bg(get_border_bg(selected, RSB_SHS_UI[2], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[RSB_SHS_UI[2]]))
                };
                let rsb_favor_low_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, RSB_SHS_UI[3], &colors))
                            .bg(get_border_bg(selected, RSB_SHS_UI[3], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[RSB_SHS_UI[3]]))
                };
                let rsb_peak_threshold_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, RSB_SHS_UI[4], &colors))
                            .bg(get_border_bg(selected, RSB_SHS_UI[4], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[RSB_SHS_UI[4]]))
                };

                let rsb_num_harmonics_content_area = rsb_num_harmonics_block.inner(rsb_num_harmonics_area);
                let rsb_harmonic_decay_content_area = rsb_harmonic_decay_block.inner(rsb_harmonic_decay_area);
                let rsb_comp_turn_point_content_area = rsb_comp_turn_point_block.inner(rsb_comp_turn_point_area);
                let rsb_favor_low_content_area = rsb_favor_low_block.inner(rsb_favor_low_area);
                let rsb_peak_threshold_content_area = rsb_peak_threshold_block.inner(rsb_peak_threshold_area);

                frame.render_widget(rsb_num_harmonics_block, rsb_num_harmonics_area);
                frame.render_widget(rsb_harmonic_decay_block, rsb_harmonic_decay_area);
                frame.render_widget(rsb_comp_turn_point_block, rsb_comp_turn_point_area);
                frame.render_widget(rsb_favor_low_block, rsb_favor_low_area);
                frame.render_widget(rsb_peak_threshold_block, rsb_peak_threshold_area);

                frame.render_widget(rsb_num_harmonics_widget, rsb_num_harmonics_content_area);
                frame.render_widget(rsb_harmonic_decay_widget, rsb_harmonic_decay_content_area);
                frame.render_widget(rsb_comp_turn_point_widget, rsb_comp_turn_point_content_area);
                frame.render_widget(rsb_favor_low_widget, rsb_favor_low_content_area);
                frame.render_widget(rsb_peak_threshold_widget, rsb_peak_threshold_content_area);
            } else if detect_mode == 1 {
                // Peak Picking
            } else if detect_mode == 2 {
                if cursor_x == Some(3) {
                    cursor_y = cursor_y.clamp(0, RSB_TWM_UI.len() - 1);
                    selected = RSB_TWM_UI[cursor_y];
                }
                // Two-Way Mismatch
                let rsb_predicted_harms_widget = {
                    Paragraph::new(format!("{}", twm_predicted_harms.get()))
                        .style(Style::default()
                            .fg(get_text_fg(selected, RSB_TWM_UI[0], &colors))
                            .bg(get_text_bg(selected, RSB_TWM_UI[0], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, RSB_TWM_UI[0], &colors))
                                .bg(get_border_bg(selected, RSB_TWM_UI[0], &colors))))
                };
                let rsb_measured_peaks_widget = {
                    Paragraph::new(format!("{}", twm_measured_peaks.get()))
                        .style(Style::default()
                            .fg(get_text_fg(selected, RSB_TWM_UI[1], &colors))
                            .bg(get_text_bg(selected, RSB_TWM_UI[1], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, RSB_TWM_UI[1], &colors))
                                .bg(get_border_bg(selected, RSB_TWM_UI[1], &colors))))
                };
                let rsb_freq_penalty_widget = {
                    Paragraph::new(format!("{}", twm_freq_penalty.get()))
                        .style(Style::default()
                            .fg(get_text_fg(selected, RSB_TWM_UI[2], &colors))
                            .bg(get_text_bg(selected, RSB_TWM_UI[2], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, RSB_TWM_UI[2], &colors))
                                .bg(get_border_bg(selected, RSB_TWM_UI[2], &colors))))
                };
                let rsb_amp_weight_widget = {
                    Paragraph::new(format!("{}", twm_amp_weight.get()))
                        .style(Style::default()
                            .fg(get_text_fg(selected, RSB_TWM_UI[3], &colors))
                            .bg(get_text_bg(selected, RSB_TWM_UI[3], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, RSB_TWM_UI[3], &colors))
                                .bg(get_border_bg(selected, RSB_TWM_UI[3], &colors))))
                };
                let rsb_freq_weight_widget = {
                    Paragraph::new(format!("{}", twm_freq_weight.get()))
                        .style(Style::default()
                            .fg(get_text_fg(selected, RSB_TWM_UI[4], &colors))
                            .bg(get_text_bg(selected, RSB_TWM_UI[4], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, RSB_TWM_UI[4], &colors))
                                .bg(get_border_bg(selected, RSB_TWM_UI[4], &colors))))
                };
                let rsb_error_ratio_widget = {
                    Paragraph::new(format!("{}", twm_error_ratio.get()))
                        .style(Style::default()
                            .fg(get_text_fg(selected, RSB_TWM_UI[5], &colors))
                            .bg(get_text_bg(selected, RSB_TWM_UI[5], &colors))
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside)
                            .border_style(Style::default()
                                .fg(get_border_fg(selected, RSB_TWM_UI[5], &colors))
                                .bg(get_border_bg(selected, RSB_TWM_UI[5], &colors))))
                };

                let [rsb_predicted_harms_area, _rsb_spacer1, rsb_measured_peaks_area, _rsb_spacer2, rsb_freq_penalty_area, _rsb_spacer3, rsb_amp_weight_area, _rsb_spacer4, rsb_freq_weight_area, _rsb_space5, rsb_error_ratio_area] = {
                    Layout::vertical([
                        Constraint::Length((rsb_predicted_harms_widget.line_count(right_sidebar_width - 2) + 2) as u16),
                        Constraint::Length(1),
                        Constraint::Length((rsb_measured_peaks_widget.line_count(right_sidebar_width - 2) + 2) as u16),
                        Constraint::Length(1),
                        Constraint::Length((rsb_freq_penalty_widget.line_count(right_sidebar_width - 2) + 2) as u16),
                        Constraint::Length(1),
                        Constraint::Length((rsb_amp_weight_widget.line_count(right_sidebar_width - 2) + 2) as u16),
                        Constraint::Length(1),
                        Constraint::Length((rsb_freq_weight_widget.line_count(right_sidebar_width - 2) + 2) as u16),
                        Constraint::Length(1),
                        Constraint::Length((rsb_error_ratio_widget.line_count(right_sidebar_width - 2) + 2) as u16),
                    ]).flex(Flex::Start)
                        .areas(right_sidebar_area)
                };

                let rsb_predicted_harms_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, RSB_TWM_UI[0], &colors))
                            .bg(get_border_bg(selected, RSB_TWM_UI[0], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[RSB_TWM_UI[0]]))
                };
                let rsb_measured_peaks_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, RSB_TWM_UI[1], &colors))
                            .bg(get_border_bg(selected, RSB_TWM_UI[1], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[RSB_TWM_UI[1]]))
                };
                let rsb_freq_penalty_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, RSB_TWM_UI[2], &colors))
                            .bg(get_border_bg(selected, RSB_TWM_UI[2], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[RSB_TWM_UI[2]]))
                };
                let rsb_amp_weight_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, RSB_TWM_UI[3], &colors))
                            .bg(get_border_bg(selected, RSB_TWM_UI[3], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[RSB_TWM_UI[3]]))
                };
                let rsb_freq_weight_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, RSB_TWM_UI[4], &colors))
                            .bg(get_border_bg(selected, RSB_TWM_UI[4], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[RSB_TWM_UI[4]]))
                };
                let rsb_error_ratio_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(get_border_fg(selected, RSB_TWM_UI[5], &colors))
                            .bg(get_border_bg(selected, RSB_TWM_UI[5], &colors)))
                        .title(format!(" {} ", UI_ID_TO_NAMES[RSB_TWM_UI[5]]))
                };

                let rsb_predicted_harms_content_area = rsb_predicted_harms_block.inner(rsb_predicted_harms_area);
                let rsb_measured_peaks_content_area = rsb_measured_peaks_block.inner(rsb_measured_peaks_area);
                let rsb_freq_penalty_content_area = rsb_freq_penalty_block.inner(rsb_freq_penalty_area);
                let rsb_amp_weight_content_area = rsb_amp_weight_block.inner(rsb_amp_weight_area);
                let rsb_freq_weight_content_area = rsb_freq_weight_block.inner(rsb_freq_weight_area);
                let rsb_error_ratio_content_area = rsb_error_ratio_block.inner(rsb_error_ratio_area);

                frame.render_widget(rsb_predicted_harms_block, rsb_predicted_harms_area);
                frame.render_widget(rsb_measured_peaks_block, rsb_measured_peaks_area);
                frame.render_widget(rsb_freq_penalty_block, rsb_freq_penalty_area);
                frame.render_widget(rsb_amp_weight_block, rsb_amp_weight_area);
                frame.render_widget(rsb_freq_weight_block, rsb_freq_weight_area);
                frame.render_widget(rsb_error_ratio_block, rsb_error_ratio_area);

                frame.render_widget(rsb_predicted_harms_widget, rsb_predicted_harms_content_area);
                frame.render_widget(rsb_measured_peaks_widget, rsb_measured_peaks_content_area);
                frame.render_widget(rsb_freq_penalty_widget, rsb_freq_penalty_content_area);
                frame.render_widget(rsb_amp_weight_widget, rsb_amp_weight_content_area);
                frame.render_widget(rsb_freq_weight_widget, rsb_freq_weight_content_area);
                frame.render_widget(rsb_error_ratio_widget, rsb_error_ratio_content_area);
            }

            // Render Tooltips
            {
                let [tooltips_area] = {
                    Layout::horizontal([
                        Constraint::Length(tooltips_width),
                    ]).flex(Flex::Center)
                        .areas(bottom_app_area)
                };

                let tooltips_block1 = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default()
                            .fg(colors["blue"])
                            .bg(colors["green"]))
                        .title(" Tooltips ")
                };
                let tooltips_block2 = {
                    Block::default()
                        .borders(Borders::LEFT | Borders::RIGHT)
                        .border_type(BorderType::QuadrantOutside)
                        .border_style(Style::default()
                            .fg(colors["blue"])
                            .bg(colors["green"]))
                };

                if selected != "None" {
                    let tooltips_block2_area = tooltips_block1.inner(tooltips_area);
                    let tooltips_content_area = tooltips_block2.inner(tooltips_block2_area);

                    let mut tooltips_info_lines: Vec<Line> = Vec::new();
                    tooltips_info_lines.push({
                        Line::from(Span::styled(UI_ID_TO_NAMES[selected],
                                                Style::default()
                                                    .fg(colors["blue"])
                                                    .add_modifier(Modifier::UNDERLINED)))
                    }); // Name
                    if selected == PITCH_DISPLAY_UI[0] || selected == VOL_DISPLAY_UI[0] {
                        tooltips_info_lines.push({
                            Line::from(Span::styled("Not Modifiable",
                                                    Style::default()
                                                        .fg(colors["grey"])))
                        }); // No Modification
                    } else {
                        tooltips_info_lines.push({
                            Line::from(Span::styled("Arrow Up / Down OR Scroll Wheel",
                                                    Style::default().fg(colors["blue"])))
                        }); // Modification Keys
                        tooltips_info_lines.push({
                            Line::from(Span::styled({
                                                        if selected == LSB_UI[0] {
                                                            format!("Range: [{min_pos_freq}, {max_freq_bound}]")
                                                        } else if selected == LSB_UI[1] {
                                                            format!("Range: [{min_freq_bound}, {max_pos_freq}]")
                                                        } else if selected == LSB_UI[2] {
                                                            format!("Range: {}", get_bounds_repr(vol_filter.lower_bound(), vol_filter.upper_bound()))
                                                        } else if selected == LSB_UI[3] {
                                                            format!("Range: [0, {}]", DETECT_METHODS.len())
                                                        } else if selected == LSB_UI[4] {
                                                            format!("Range: {}", get_bounds_repr(vol_steepness.lower_bound(), vol_steepness.upper_bound()))
                                                        } else if selected == RSB_SHS_UI[0] {
                                                            format!("Range: {}", get_bounds_repr(shs_num_harmonics.lower_bound(), shs_num_harmonics.upper_bound()))
                                                        } else if selected == RSB_SHS_UI[1] {
                                                            format!("Range: {}", get_bounds_repr(shs_harmonic_decay.lower_bound(), shs_harmonic_decay.upper_bound()))
                                                        } else if selected == RSB_SHS_UI[2] {
                                                            format!("Range: {}", get_bounds_repr(shs_comp_turn_point.lower_bound(), shs_comp_turn_point.upper_bound()))
                                                        } else if selected == RSB_SHS_UI[3] {
                                                            String::from("Range: Yes / No")
                                                        } else if selected == RSB_SHS_UI[4] {
                                                            format!("Range: {}", get_bounds_repr(shs_peak_threshold.lower_bound(), shs_peak_threshold.upper_bound()))
                                                        } else if selected == RSB_TWM_UI[0] {
                                                            format!("Range: {}", get_bounds_repr(twm_predicted_harms.lower_bound(), twm_predicted_harms.upper_bound()))
                                                        } else if selected == RSB_TWM_UI[1] {
                                                            format!("Range: {}", get_bounds_repr(twm_measured_peaks.lower_bound(), twm_measured_peaks.upper_bound()))
                                                        } else if selected == RSB_TWM_UI[2] {
                                                            format!("Range: {}", get_bounds_repr(twm_freq_penalty.lower_bound(), twm_freq_penalty.upper_bound()))
                                                        } else if selected == RSB_TWM_UI[3] {
                                                            format!("Range: {}", get_bounds_repr(twm_amp_weight.lower_bound(), twm_amp_weight.upper_bound()))
                                                        } else if selected == RSB_TWM_UI[4] {
                                                            format!("Range: {}", get_bounds_repr(twm_freq_weight.lower_bound(), twm_freq_weight.upper_bound()))
                                                        } else if selected == RSB_TWM_UI[5] {
                                                            format!("Range: {}", get_bounds_repr(twm_error_ratio.lower_bound(), twm_error_ratio.upper_bound()))
                                                        } else {
                                                            String::from("")
                                                        }
                                                    },
                                                    Style::default().fg(colors["blue"])))
                        }); // Range
                        if UI_IDS_W_SUGGESTED.contains(&selected) {
                            tooltips_info_lines.push(
                                Line::from(Span::styled({
                                                            if selected == LSB_UI[2] {
                                                                "Suggested: [0.003, 0.03]"
                                                            } else if selected == LSB_UI[4] {
                                                                "Suggested: [2, 4]"
                                                            } else if selected == RSB_SHS_UI[0] {
                                                                "Suggested: [3, 8]"
                                                            } else if selected == RSB_SHS_UI[1] {
                                                                "Suggested: [0.7, 0.9]"
                                                            } else if selected == RSB_SHS_UI[2] {
                                                                "Suggested: [10.0, 1000.0]"
                                                            } else if selected == RSB_SHS_UI[3] {
                                                                "Suggested: Yes"
                                                            } else if selected == RSB_SHS_UI[4] {
                                                                "Suggested: [0.8, 0.95]"
                                                            } else if selected == RSB_TWM_UI[0] || selected == RSB_TWM_UI[1] {
                                                                "Suggested: [8, 15]"
                                                            } else if selected == RSB_TWM_UI[2] {
                                                                "Suggested: [0.25, 0.75]"
                                                            } else if selected == RSB_TWM_UI[3] {
                                                                "Suggested: [0.05, 0.2]"
                                                            } else if selected == RSB_TWM_UI[4] {
                                                                "Suggested: [0.2, 0.8]"
                                                            } else if selected == RSB_TWM_UI[5] {
                                                                "Suggested: [0.2, 0.6]"
                                                            } else {
                                                                ""
                                                            }
                                                        },
                                                        Style::default().fg(colors["blue"])))
                            );
                            tooltips_info_lines.push(Line::from(""));
                            let mut within = 0;
                            if UI_IDS_W_DIRECTION.contains(&selected) {
                                tooltips_info_lines.push(Line::from(Span::styled({
                                                                                    if selected == LSB_UI[2] {
                                                                                        if vol_filter < 0.003 {
                                                                                            if vol_filter.shift_down().is_ok() {
                                                                                                within = 1;
                                                                                                "BELOW SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MINIMUM"
                                                                                            }
                                                                                        } else if vol_filter > 0.03 {
                                                                                            if vol_filter.shift_up().is_ok() {
                                                                                                within = 1;
                                                                                                "ABOVE SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MAXIMUM"
                                                                                            }
                                                                                        } else {
                                                                                            "WITHIN SUGGESTED RANGE"
                                                                                        }
                                                                                    } else if selected == LSB_UI[4] {
                                                                                        if vol_steepness < 2 {
                                                                                            if vol_steepness.shift_down().is_ok() {
                                                                                                within = 1;
                                                                                                "BELOW SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MINIMUM"
                                                                                            }
                                                                                        } else if vol_steepness > 4 {
                                                                                            if vol_steepness.shift_up().is_ok() {
                                                                                                within = 1;
                                                                                                "ABOVE SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MAXIMUM"
                                                                                            }
                                                                                        } else {
                                                                                            "WITHIN SUGGESTED RANGE"
                                                                                        }
                                                                                    } else if selected == RSB_SHS_UI[0] {
                                                                                        if shs_num_harmonics < 3 {
                                                                                            if shs_num_harmonics.shift_down().is_ok() {
                                                                                                within = 1;
                                                                                                "BELOW SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MINIMUM"
                                                                                            }
                                                                                        } else if shs_num_harmonics > 8 {
                                                                                            if shs_num_harmonics.shift_up().is_ok() {
                                                                                                within = 1;
                                                                                                "ABOVE SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MAXIMUM"
                                                                                            }
                                                                                        } else {
                                                                                            "WITHIN SUGGESTED RANGE"
                                                                                        }
                                                                                    } else if selected == RSB_SHS_UI[1] {
                                                                                        if shs_harmonic_decay < 0.7 {
                                                                                            if shs_harmonic_decay.shift_down().is_ok() {
                                                                                                within = 1;
                                                                                                "BELOW SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MINIMUM"
                                                                                            }
                                                                                        } else if shs_harmonic_decay > 0.9 {
                                                                                            if shs_harmonic_decay.shift_up().is_ok() {
                                                                                                within = 1;
                                                                                                "ABOVE SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MAXIMUM"
                                                                                            }
                                                                                        } else {
                                                                                            "WITHIN SUGGESTED RANGE"
                                                                                        }
                                                                                    } else if selected == RSB_SHS_UI[2] {
                                                                                        if shs_comp_turn_point < 10.0 {
                                                                                            if shs_comp_turn_point.shift_down().is_ok() {
                                                                                                within = 1;
                                                                                                "BELOW SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MINIMUM"
                                                                                            }
                                                                                        } else if shs_comp_turn_point > 1000.0 {
                                                                                            if shs_comp_turn_point.shift_up().is_ok() {
                                                                                                within = 1;
                                                                                                "ABOVE SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MAXIMUM"
                                                                                            }
                                                                                        } else {
                                                                                            "WITHIN SUGGESTED RANGE"
                                                                                        }
                                                                                    } else if selected == RSB_SHS_UI[4] {
                                                                                        if shs_peak_threshold < 0.8 {
                                                                                            if shs_peak_threshold.shift_down().is_ok() {
                                                                                                within = 1;
                                                                                                "BELOW SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MINIMUM"
                                                                                            }
                                                                                        } else if shs_peak_threshold > 0.95 {
                                                                                            if shs_peak_threshold.shift_up().is_ok() {
                                                                                                within = 1;
                                                                                                "ABOVE SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MAXIMUM"
                                                                                            }
                                                                                        } else {
                                                                                            "WITHIN SUGGESTED RANGE"
                                                                                        }
                                                                                    } else if selected == RSB_TWM_UI[0] {
                                                                                        if twm_predicted_harms < 8 {
                                                                                            if twm_predicted_harms.shift_down().is_ok() {
                                                                                                within = 1;
                                                                                                "BELOW SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MINIMUM"
                                                                                            }
                                                                                        } else if twm_predicted_harms > 15 {
                                                                                            if twm_predicted_harms.shift_up().is_ok() {
                                                                                                within = 1;
                                                                                                "ABOVE SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MAXIMUM"
                                                                                            }
                                                                                        } else {
                                                                                            "WITHIN SUGGESTED RANGE"
                                                                                        }
                                                                                    } else if selected == RSB_TWM_UI[1] {
                                                                                        if twm_measured_peaks < 8 {
                                                                                            if twm_measured_peaks.shift_down().is_ok() {
                                                                                                within = 1;
                                                                                                "BELOW SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MINIMUM"
                                                                                            }
                                                                                        } else if twm_measured_peaks > 15 {
                                                                                            if twm_measured_peaks.shift_up().is_ok() {
                                                                                                within = 1;
                                                                                                "ABOVE SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MAXIMUM"
                                                                                            }
                                                                                        } else {
                                                                                            "WITHIN SUGGESTED RANGE"
                                                                                        }
                                                                                    } else if selected == RSB_TWM_UI[2] {
                                                                                        if twm_freq_penalty < 0.25 {
                                                                                            if twm_freq_penalty.shift_down().is_ok() {
                                                                                                within = 1;
                                                                                                "BELOW SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MINIMUM"
                                                                                            }
                                                                                        } else if twm_freq_penalty > 0.75 {
                                                                                            if twm_freq_penalty.shift_up().is_ok() {
                                                                                                within = 1;
                                                                                                "ABOVE SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MAXIMUM"
                                                                                            }
                                                                                        } else {
                                                                                            "WITHIN SUGGESTED RANGE"
                                                                                        }
                                                                                    } else if selected == RSB_TWM_UI[3] {
                                                                                        if twm_amp_weight < 0.05 {
                                                                                            if twm_amp_weight.shift_down().is_ok() {
                                                                                                within = 1;
                                                                                                "BELOW SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MINIMUM"
                                                                                            }
                                                                                        } else if twm_amp_weight > 0.2 {
                                                                                            if twm_amp_weight.shift_up().is_ok() {
                                                                                                within = 1;
                                                                                                "ABOVE SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MAXIMUM"
                                                                                            }
                                                                                        } else {
                                                                                            "WITHIN SUGGESTED RANGE"
                                                                                        }
                                                                                    } else if selected == RSB_TWM_UI[4] {
                                                                                        if twm_freq_weight < 0.2 {
                                                                                            if twm_freq_weight.shift_down().is_ok() {
                                                                                                within = 1;
                                                                                                "BELOW SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MINIMUM"
                                                                                            }
                                                                                        } else if twm_freq_weight > 0.8 {
                                                                                            if twm_freq_weight.shift_up().is_ok() {
                                                                                                within = 1;
                                                                                                "ABOVE SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MAXIMUM"
                                                                                            }
                                                                                        } else {
                                                                                            "WITHIN SUGGESTED RANGE"
                                                                                        }
                                                                                    } else if selected == RSB_TWM_UI[5] {
                                                                                        if twm_error_ratio < 0.2 {
                                                                                            if twm_error_ratio.shift_down().is_ok() {
                                                                                                within = 1;
                                                                                                "BELOW SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MINIMUM"
                                                                                            }
                                                                                        } else if twm_error_ratio > 0.6 {
                                                                                            if twm_error_ratio.shift_up().is_ok() {
                                                                                                within = 1;
                                                                                                "ABOVE SUGGESTED RANGE"
                                                                                            } else {
                                                                                                within = 2;
                                                                                                "AT MAXIMUM"
                                                                                            }
                                                                                        } else {
                                                                                            "WITHIN SUGGESTED RANGE"
                                                                                        }
                                                                                    } else {
                                                                                        ""
                                                                                    }
                                                                                },
                                                                                 Style::default()
                                                                                    .fg(if within == 0 {
                                                                                        colors["green"]
                                                                                    } else if within == 1 {
                                                                                        colors["yellow"]
                                                                                    } else {
                                                                                        colors["red"]
                                                                                    }))))
                            } else {
                                tooltips_info_lines.push(Line::from(Span::styled({
                                                                                    if shs_favor_low == false {
                                                                                        within = 1;
                                                                                        "NOT SUGGESTED VALUE"
                                                                                    } else {
                                                                                        "SUGGESTED VALUE"
                                                                                    }
                                                                                },
                                                                                 Style::default()
                                                                                    .fg(if within == 0 {
                                                                                        colors["green"]
                                                                                    } else if within == 1 {
                                                                                        colors["yellow"]
                                                                                    } else {
                                                                                        colors["red"]
                                                                                    }))))
                            }
                        } else {
                            if UI_IDS_W_DIRECTION.contains(&selected) {
                                tooltips_info_lines.push(Line::from(""));
                                let mut within = 0;

                                tooltips_info_lines.push(Line::from(Span::styled({
                                                                                    if selected == LSB_UI[0] {
                                                                                        if min_freq.shift_down_bounded(min_pos_freq).is_err() {
                                                                                            within = 2;
                                                                                            "AT MINIMUM"
                                                                                        } else if min_freq.shift_up_bounded(max_freq_bound).is_err() {
                                                                                            within = 2;
                                                                                            "AT MAXIMUM"
                                                                                        } else {
                                                                                            "WITHIN RANGE"
                                                                                        }
                                                                                    } else if selected == LSB_UI[1] {
                                                                                        if max_freq.shift_down_bounded(min_freq_bound).is_err() {
                                                                                            within = 2;
                                                                                            "AT MINIMUM"
                                                                                        } else if max_freq.shift_up_bounded(max_pos_freq).is_err() {
                                                                                            within = 2;
                                                                                            "AT MAXIMUM"
                                                                                        } else {
                                                                                            "WITHIN RANGE"
                                                                                        }
                                                                                    } else {
                                                                                        ""
                                                                                    }
                                                                                },
                                                                                 Style::default()
                                                                                    .fg(if within == 0 {
                                                                                        colors["green"]
                                                                                    } else if within == 1 {
                                                                                        colors["yellow"]
                                                                                    } else {
                                                                                        colors["red"]
                                                                                    }))))
                            }
                        } // Suggested Range & Range Errors
                    } // Modification

                    let tooltip_info_width = tooltips_info_lines.iter().map(|line| line.width()).max().unwrap() as u16;

                    let [_tooltips_spacer1, tooltips_info_horizontal, _tooltips_spacer2, tooltips_seperator_area, _tooltips_spacer3, tooltips_desc_horizontal, _tooltips_spacer4] = {
                        Layout::horizontal([
                            Constraint::Length(2),
                            Constraint::Length(tooltip_info_width),
                            Constraint::Length(2),
                            Constraint::Length(2),
                            Constraint::Length(2),
                            Constraint::Fill(1),
                            Constraint::Length(2),
                        ]).flex(Flex::Center)
                            .areas(tooltips_content_area)
                    };

                    let tooltips_info_widget = Paragraph::new(tooltips_info_lines);
                    let tooltips_info_height = tooltips_info_widget.line_count(tooltip_info_width);

                    let [tooltips_info_area] = {
                        Layout::vertical([
                            Constraint::Length(tooltips_info_height as u16)
                        ]).flex(Flex::Center)
                            .areas(tooltips_info_horizontal)
                    };

                    let tooltips_seperator_widget = {
                        Paragraph::new((0..tooltips_height-2).map(|_| Line::from(Span::styled("┃┃",
                                                                                              Style::default()
                                                                                                  .fg(colors["blue"])
                                                                                                  .bg(colors["green"])))).collect::<Vec<_>>())
                    };


                    frame.render_widget(tooltips_block1, tooltips_area);
                    frame.render_widget(tooltips_block2, tooltips_block2_area);

                    frame.render_widget(tooltips_info_widget, tooltips_info_area);
                    frame.render_widget(tooltips_seperator_widget, tooltips_seperator_area);
                }
            }
        })?;

        thread::sleep(Duration::from_millis(16));

        while event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? && key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Up => {
                        if selected == LSB_UI[0] {
                            if let Ok(new_min) = min_freq.shift_up_bounded(max_freq_bound) {
                                safe_audio_msg(&out_post, AudioMessage::UpdateMinFreq(new_min))?;
                            }
                        } else if selected == LSB_UI[1] {
                            if let Ok(new_max) = max_freq.shift_up_bounded(max_pos_freq) {
                                safe_audio_msg(&out_post, AudioMessage::UpdateMaxFreq(new_max))?;
                            }
                        } else if selected == LSB_UI[2] {
                            if let Ok(new_vol_filter) = vol_filter.shift_up() {
                                vol_filter.set(new_vol_filter);
                            }
                        } else if selected == LSB_UI[3] {
                            detect_mode.set(detect_mode.shift_up());
                        } else if selected == LSB_UI[4] {
                            if let Ok(new_vol_steepness) = vol_steepness.shift_up() {
                                vol_steepness.set(new_vol_steepness);
                            }
                        } else if selected == RSB_SHS_UI[0] {
                            if let Ok(new_shs_num_harmonics) = shs_num_harmonics.shift_up() {
                                shs_num_harmonics.set(new_shs_num_harmonics);
                            }
                        } else if selected == RSB_SHS_UI[1] {
                            if let Ok(new_shs_harmonic_decay) = shs_harmonic_decay.shift_up() {
                                shs_harmonic_decay.set(new_shs_harmonic_decay);
                            }
                        } else if selected == RSB_SHS_UI[2] {
                            if let Ok(new_shs_comp_turn_point) = shs_comp_turn_point.shift_up() {
                                shs_comp_turn_point.set(new_shs_comp_turn_point);
                            }
                        } else if selected == RSB_SHS_UI[3] {
                            shs_favor_low.set(shs_favor_low.toggle());
                        } else if selected == RSB_SHS_UI[4] {
                            if let Ok(new_shs_peak_threshold) = shs_peak_threshold.shift_up() {
                                shs_peak_threshold.set(new_shs_peak_threshold);
                            }
                        } else if selected == RSB_TWM_UI[0] {
                            if let Ok(new_twm_predicted_harms) = twm_predicted_harms.shift_up() {
                                twm_predicted_harms.set(new_twm_predicted_harms);
                            }
                        } else if selected == RSB_TWM_UI[1] {
                            if let Ok(new_twm_measured_peaks) = twm_measured_peaks.shift_up() {
                                twm_measured_peaks.set(new_twm_measured_peaks);
                            }
                        } else if selected == RSB_TWM_UI[2] {
                            if let Ok(new_twm_freq_penalty) = twm_freq_penalty.shift_up() {
                                twm_freq_penalty.set(new_twm_freq_penalty);
                            }
                        } else if selected == RSB_TWM_UI[3] {
                            if let Ok(new_twm_amp_weight) = twm_amp_weight.shift_up() {
                                twm_amp_weight.set(new_twm_amp_weight);
                            }
                        } else if selected == RSB_TWM_UI[4] {
                            if let Ok(new_twm_freq_weight) = twm_freq_weight.shift_up() {
                                twm_freq_weight.set(new_twm_freq_weight);
                            }
                        } else if selected == RSB_TWM_UI[5] {
                            if let Ok(new_twm_error_ratio) = twm_error_ratio.shift_up() {
                                twm_error_ratio.set(new_twm_error_ratio);
                            }
                        }
                    }
                    KeyCode::Down => {
                        if selected == LSB_UI[0] {
                            if let Ok(new_min) = min_freq.shift_down_bounded(min_pos_freq) {
                                safe_audio_msg(&out_post, AudioMessage::UpdateMinFreq(new_min))?;
                            }
                        } else if selected == LSB_UI[1] {
                            if let Ok(new_max) = max_freq.shift_down_bounded(min_freq_bound) {
                                safe_audio_msg(&out_post, AudioMessage::UpdateMaxFreq(new_max))?;
                            }
                        } else if selected == LSB_UI[2] {
                            if let Ok(new_vol_filter) = vol_filter.shift_down() {
                                vol_filter.set(new_vol_filter);
                            }
                        } else if selected == LSB_UI[3] {
                            detect_mode.set(detect_mode.shift_down());
                        } else if selected == LSB_UI[4] {
                            if let Ok(new_vol_steepness) = vol_steepness.shift_down() {
                                vol_steepness.set(new_vol_steepness);
                            }
                        } else if selected == RSB_SHS_UI[0] {
                            if let Ok(new_shs_num_harmonics) = shs_num_harmonics.shift_down() {
                                shs_num_harmonics.set(new_shs_num_harmonics);
                            }
                        } else if selected == RSB_SHS_UI[1] {
                            if let Ok(new_shs_harmonic_decay) = shs_harmonic_decay.shift_down() {
                                shs_harmonic_decay.set(new_shs_harmonic_decay);
                            }
                        } else if selected == RSB_SHS_UI[2] {
                            if let Ok(new_shs_comp_turn_point) = shs_comp_turn_point.shift_down() {
                                shs_comp_turn_point.set(new_shs_comp_turn_point);
                            }
                        } else if selected == RSB_SHS_UI[3] {
                            shs_favor_low.set(shs_favor_low.toggle());
                        } else if selected == RSB_SHS_UI[4] {
                            if let Ok(new_shs_peak_threshold) = shs_peak_threshold.shift_down() {
                                shs_peak_threshold.set(new_shs_peak_threshold);
                            }
                        } else if selected == RSB_TWM_UI[0] {
                            if let Ok(new_twm_predicted_harms) = twm_predicted_harms.shift_down() {
                                twm_predicted_harms.set(new_twm_predicted_harms);
                            }
                        } else if selected == RSB_TWM_UI[1] {
                            if let Ok(new_twm_measured_peaks) = twm_measured_peaks.shift_down() {
                                twm_measured_peaks.set(new_twm_measured_peaks);
                            }
                        } else if selected == RSB_TWM_UI[2] {
                            if let Ok(new_twm_freq_penalty) = twm_freq_penalty.shift_down() {
                                twm_freq_penalty.set(new_twm_freq_penalty);
                            }
                        } else if selected == RSB_TWM_UI[3] {
                            if let Ok(new_twm_amp_weight) = twm_amp_weight.shift_down() {
                                twm_amp_weight.set(new_twm_amp_weight);
                            }
                        } else if selected == RSB_TWM_UI[4] {
                            if let Ok(new_twm_freq_weight) = twm_freq_weight.shift_down() {
                                twm_freq_weight.set(new_twm_freq_weight);
                            }
                        } else if selected == RSB_TWM_UI[5] {
                            if let Ok(new_twm_error_ratio) = twm_error_ratio.shift_down() {
                                twm_error_ratio.set(new_twm_error_ratio);
                            }
                        } else {
                            let _ = 0;
                        }
                    }
                    KeyCode::Char('w') => {
                        if cursor_x.is_none() {
                            cursor_x = last_cursor_x;
                            last_cursor_x = None;
                        } else {
                            cursor_y = cursor_y.saturating_sub(1);
                        }
                    }
                    KeyCode::Char('s') => {
                        if cursor_x.is_none() {
                            cursor_x = last_cursor_x;
                            last_cursor_x = None;
                        } else {
                            cursor_y += 1;
                        }
                    }
                    KeyCode::Char('a') => {
                        if let Some(cursor_x_val) = cursor_x {
                            if cursor_x_val > 0 {
                                cursor_x = Some(cursor_x_val - 1);
                            }
                        } else {
                            cursor_x = last_cursor_x;
                            last_cursor_x = None;
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Some(cursor_x_val) = cursor_x {
                            cursor_x = Some(cursor_x_val + 1);
                        } else {
                            cursor_x = last_cursor_x;
                            last_cursor_x = None;
                        }
                    }
                    KeyCode::Char(' ') => {
                        if last_cursor_x.is_some() {
                            cursor_x = last_cursor_x;
                            last_cursor_x = None;
                        } else {
                            last_cursor_x = cursor_x;
                            cursor_x = None;
                            selected = "None";
                        }
                    }
                    KeyCode::Char('r') => {
                        let new_min = min_freq.default();
                        let new_max = max_freq.default();
                        safe_audio_msg(&out_post, AudioMessage::UpdateBounds(new_min, new_max))?;

                        vol_filter.set(vol_filter.default());
                        vol_steepness.set(vol_steepness.default());

                        detect_mode.set(detect_mode.default());

                        shs_num_harmonics.set(shs_num_harmonics.default());
                        shs_harmonic_decay.set(shs_harmonic_decay.default());
                        shs_comp_turn_point.set(shs_comp_turn_point.default());
                        shs_favor_low.set(shs_favor_low.default());
                        shs_peak_threshold.set(shs_peak_threshold.default());

                        twm_predicted_harms.set(twm_predicted_harms.default());
                        twm_measured_peaks.set(twm_measured_peaks.default());
                        twm_freq_penalty.set(twm_freq_penalty.default());
                        twm_amp_weight.set(twm_amp_weight.default());
                        twm_freq_weight.set(twm_freq_penalty.default());
                        twm_error_ratio.set(twm_error_ratio.default());
                    }
                    KeyCode::Char('q') => {
                        drop(audio_stream);
                        if safe_audio_msg(&out_post, AudioMessage::Shutdown).is_err() {
                            wait = false;
                        }
                        break 'main;
                    }
                    _ => {}
                }
            }
        }
    }

    if wait {
        for msg in in_mail {
            if let OutputMessage::Shutdown = msg { break }
        }
    }

    handle.join().ok();

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}


fn setup_audio_callback(min_f: f32, max_f: f32, bins_per_octave: u32, in_post: Sender<OutputMessage>, buffer_size: BufferSize) -> Result<(Device, Stream, Sender<AudioMessage>, JoinHandle<Result<(), String>>), Box<dyn error::Error>> {
    let (device, config) = device_setup()?;

    let (out_post, handle) = start_process_thread(config.sample_rate() as f32, min_f, max_f, bins_per_octave, in_post)?;

    let stream = setup_audio_stream(&device, config, out_post.clone(), buffer_size)?;

    Ok((device, stream, out_post, handle))
}

fn setup_audio_stream(device: &Device, config: SupportedStreamConfig, out_post: Sender<AudioMessage>, buffer_size: BufferSize) -> Result<Stream, Box<dyn error::Error>> {
    //let err_fn = |err| eprintln!("An error occurred on the audio stream: {err}");
    let err_fn = |_| {};

    let mut stream_config: StreamConfig = config.into();
    stream_config.buffer_size = buffer_size;

    let stream = match config.sample_format() {
        SampleFormat::F32 => device.build_input_stream(
            stream_config,
            move |data: &[f32], _| {
                match safe_audio_msg(&out_post, AudioMessage::AudioChunk(data.to_vec())) {
                    Ok(()) => {},
                    Err(_desc) => { /* println!("{}", desc) */ },
                }

            },
            err_fn,
            None,
        )?,
        sform => Err(format!("Unsuported sample format: {sform}"))?
    };

    Ok(stream)
}

fn device_setup() -> Result<(Device, SupportedStreamConfig), Box<dyn error::Error>> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("Failed to find a default input audio device (microphone)")?;
    let config: SupportedStreamConfig = device.default_input_config()?;

    Ok((device, config))
}

pub fn safe_audio_msg(post: &Sender<AudioMessage>, msg: AudioMessage) -> Result<(), String> {
    if let Err(_send_err) = post.send(msg) {
        eprintln!("Processing thread disconnected, restarting");
        return Err(String::from("Processing thread disconnected, restarting"));
    }
    Ok(())
}

fn normalize_disp_str(disp_str: &str) -> String {
    let mut norm_lines = String::new();
    for line in disp_str.lines() {
        if !line.is_empty() {
            norm_lines += line;
            norm_lines += "\n";
        }
    }
    let norm_lines = norm_lines;


    let mut longest = 0;
    for line in norm_lines.lines() {
        if line.len() > longest {
            longest = line.len();
        }
    }
    let longest = longest;

    let mut norm_len = String::new();
    for line in norm_lines.lines() {
        norm_len += line;
        if line.len() < longest {
            norm_len += &*" ".repeat(longest - line.len());
        }
        norm_len += "\n";
    }

    norm_len
}

fn horizontal_concat(let_str: &str, oct_str: &str) -> String {
    let mut out_line = String::new();
    for (let_line, oct_line) in zip(let_str.lines(), oct_str.lines()) {
        out_line += let_line;
        out_line += " ";
        out_line += oct_line;
        out_line += "\n";
    }
    out_line
}

fn freq_to_inds(freq: f32) -> Result<(usize, usize), (bool, String)> {
    let mut old_diff: f32 = f32::INFINITY;
    let mut last_oct: usize = 0;
    let mut last_let: usize = 0;
    let mut start: bool = true;

    for octave in 0usize..10 {
        for (letter_ind, letter) in FREQUENCIES.iter().enumerate() {
            let new_diff = freq - letter * 2.0f32.powf(octave as f32);
            if new_diff <= 0.0 {
                let spec_diff = freq - FREQUENCIES[11] * 2.0f32.powf(-1.0);
                if start && (spec_diff < 0.0 || new_diff.abs() > spec_diff.abs()) {
                    return Err((false, String::from("Number too Low to Match")));
                }
                return if new_diff.abs() <= old_diff {
                    Ok((letter_ind, octave))
                } else {
                    Ok((last_let, last_oct))
                }
            }
            old_diff = new_diff;
            last_oct = octave;
            last_let = letter_ind;
            if start { start = false; }
        }
    }

    Err((true, String::from("Number too High to Match")))
}

fn create_bar(level: f32, width: usize, height: usize) -> Vec<Line<'static>> {
    let blocks = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    let total_steps = height * 8;
    let full_steps = (level.clamp(0.0, 1.0) * total_steps as f32).round() as usize;

    let full_blocks = full_steps / 8;
    let last_block = full_steps % 8;

    let mut lines = Vec::new();

    for row in (0..height).rev() {
        let new_block = if row < full_blocks {
            blocks[8].repeat(width)
        } else if row == full_blocks {
            blocks[last_block].repeat(width)
        } else {
            blocks[0].repeat(width)
        };

        let color = if row >= height / 2 {
            Color::Green
        } else if row >= height / 4 {
            Color::Yellow
        } else {
            Color::Red
        };

        lines.push(Line::from(Span::styled(new_block, Style::default().fg(color))));
    }

    lines
}

#[inline(always)]
fn normalize(input: f32, midpoint: f32, curve: u16) -> f32 {
    input.max(0.0).powf(curve as f32) / (midpoint.powf(curve as f32) + input.max(0.0).powf(curve as f32))
}

#[inline(always)]
fn get_color((r, g, b): (u8, u8, u8)) -> Color {
    Color::Rgb(r, g, b)
}

#[inline(always)]
fn get_text_fg(selected: &str, id: &str, colors: &HashMap<&str, Color>) -> Color {
    if selected == id {colors["green"]} else {colors["blue"]}
}

#[inline(always)]
fn get_text_bg(selected: &str, id: &str, colors: &HashMap<&str, Color>) -> Color {
    if selected == id {colors["blue"]} else {Color::Reset}
}

#[inline(always)]
fn get_border_fg(selected: &str, id: &str, colors: &HashMap<&str, Color>) -> Color {
    if selected == id {colors["lightgreen"]} else {colors["blue"]}
}

#[inline(always)]
fn get_border_bg(selected: &str, id: &str, colors: &HashMap<&str, Color>) -> Color {
    if selected == id {colors["darkblue"]} else {colors["green"]}
}

fn get_bounds_repr<T>(lower: Bound<T>, upper: Bound<T>) -> String where T: Display {
    let lower_str = match lower {
        Bound::Included(num) => format!("[{num}"),
        Bound::Excluded(num) => format!("({num}"),
        Bound::Unbounded => String::from("(∞"),
    };
    let upper_str =  match upper {
        Bound::Included(num) => format!("{num}]"),
        Bound::Excluded(num) => format!("{num})"),
        Bound::Unbounded => String::from("∞)"),
    };

    format!("{lower_str}, {upper_str}")
}