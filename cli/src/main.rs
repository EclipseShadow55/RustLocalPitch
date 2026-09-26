mod parameters;

use std::{
    iter::{zip},
    io,
    sync::mpsc::{channel, Sender},
    time::{Duration, Instant},
    thread,
    thread::{JoinHandle},
    error,
    ops::{Bound, RangeBounds},
    f32::consts as f32consts
};
use num_traits::{Num, Zero};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Flex},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
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

use parameters::{
    Parameter,
    BoundedParameter,
    LinearParam,
    ExponentialParam,
    CyclicParam,
    BoolParam,
    BoundedParam,
};

/* [ ] main()
        [X] Open Terminal and put into UI mode or whatever
        [X] Create inbox
        [ ] Start processing thread and get back the out-post
        [ ] Start audio stream
        [ ] Every frame:
            [ ] Get terminal event like key presses and process them
                [ ] Up / Down = raise / lower minimum_frequency by 1 bin
                [ ] Right / Left = raise / lower maximum_frequency by 1 bin
                [ ] W / D = raise / lower bins_per_octave by 1 (display error if trying to go to 0 or below)
                [ ] Q = shutdown
            [ ] Process messages from inbox
                [ ] OutputMessage::NewFreqMax
                [ ] OutputMessage::NewFreqFrame
                [ ] OutputMessage::ConfirmMinFreq
                [ ] OutputMessage::ConfirmMaxFreq
                [ ] OutputMessage::ConfirmBinsPerOctave
                [ ] OutputMessage::ReturnFreqs
                [ ] OutputMessage::Shutdown
                [ ] OutputMessage::Confirm
        [ ] On quit:
            [ ] close audio stream
            [ ] send AudioMessage:Shutdown
            [ ] keep recieving messages until OutputMessage::Shutdown received
            [ ] clean up and exit
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


fn main() -> Result<(), Box<dyn error::Error>> {
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

    let mut min_bound: f32 = min_freq.shift_up();
    let mut max_bound: f32 = max_freq.shift_down();
    let mut bins = get_bins(min_freq.get(), max_freq.get(), 12);

    let (_device, audio_stream, out_post, handle) = setup_audio_callback(min_freq.get(), max_freq.get(), 12, in_post, BufferSize::Fixed(512))?;

    // Volume Parameters
    let mut vol_filter = {
        BoundedParam::new(
            LinearParam::new(
                0.005f32,
                0.001),
            Bound::Included(0.0),
            Bound::Unbounded
        )
    };
    let mut vol_steepness = {
        BoundedParam::new(
            LinearParam::new(
                2,
                1),
            Bound::Excluded(1),
            Bound::Unbounded
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
            Bound::Unbounded
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
            Bound::Unbounded
        )
    };
    let mut shs_favor_low = BoolParam::new(true);
    let mut shs_peak_threshold = { // 0.8 - 0.95
        BoundedParam::new(
            LinearParam::new(0.9f32, 0.02),
            Bound::Included(0.0),
            Bound::Excluded(1.0)
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
            Bound::Unbounded,
        )
    };
    let mut twm_freq_weight = { // 0.2 - 0.8
        BoundedParam::new(
            ExponentialParam::new(0.4f32, 2.0),
            Bound::Included(0.0625),
            Bound::Unbounded,
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
    let mut cursor_position: (usize, usize) = (0, 0);

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
                    min_bound = min_freq.shift_up();
                    bins = get_bins(min_freq.get(), max_freq.get(), 12);
                }
                OutputMessage::ConfirmMaxFreq(new_max, accepted, _desc) if accepted => {
                    max_freq.set(new_max);
                    max_bound = max_freq.shift_down();
                    bins = get_bins(min_freq.get(), max_freq.get(), 12);
                }
                OutputMessage::ConfirmBounds(new_min, new_max, accepted, _desc) if accepted => {
                    min_freq.set(new_min);
                    max_freq.set(new_max);
                    min_bound = min_freq.shift_up();
                    max_bound = max_freq.shift_down();
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

            let main_app_height = 35;
            let left_sidebar_width = 35;
            let pitch_display_width = 81;
            let vol_display_width = 10;
            let right_sidebar_width = 35;

            let [vertical_center] = {
                Layout::vertical([Constraint::Length(main_app_height)])
                    .flex(Flex::Center)
                    .areas(screen_area)
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
                    .areas(vertical_center)
            };

            // Render Pitch Display
            let pitch_display_block1 = {
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                    .title(" Pitch Detector ")
            };
            let pitch_display_block2 = {
                Block::default()
                    .borders(Borders::LEFT | Borders::RIGHT)
                    .border_type(BorderType::QuadrantOutside)
                    .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
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

            let mut pitch_display_content_widget = Paragraph::new(format!("{}", current_note)).alignment(Alignment::Center);
            if current_note == no_note_ref {
                pitch_display_content_widget = pitch_display_content_widget.style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
            } else {
                pitch_display_content_widget = pitch_display_content_widget.style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
            }

            frame.render_widget(pitch_display_block1, pitch_display_area);
            frame.render_widget(pitch_display_block2, pitch_display_block2_area);

            frame.render_widget(pitch_display_content_widget, pitch_display_content_horizontal);

            // Render Left Sidebar
            let lsb_min_freq_widget = {
                Paragraph::new(format!("{}", min_freq.get()))
                    .style(Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false })
                    .block(Block::default()
                        .borders(Borders::LEFT | Borders::RIGHT)
                        .border_type(BorderType::QuadrantOutside))
            };
            let lsb_max_freq_widget = {
                Paragraph::new(format!("{}", max_freq.get()))
                    .style(Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false })
                    .block(Block::default()
                        .borders(Borders::LEFT | Borders::RIGHT)
                        .border_type(BorderType::QuadrantOutside))
            };
            let lsb_str_filter_widget = {
                Paragraph::new(format!("{}", vol_filter.get()))
                    .style(Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false })
                    .block(Block::default()
                        .borders(Borders::LEFT | Borders::RIGHT)
                        .border_type(BorderType::QuadrantOutside))
            };
            let lsb_detect_mode_widget = {
                Paragraph::new(DETECT_METHODS[detect_mode.get()])
                    .style(Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false })
                    .block(Block::default()
                        .borders(Borders::LEFT | Borders::RIGHT)
                        .border_type(BorderType::QuadrantOutside))
            };
            let lsb_vol_steepness_widget = {
                Paragraph::new(format!("{}", vol_steepness.get()))
                    .style(Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false })
                    .block(Block::default()
                        .borders(Borders::LEFT | Borders::RIGHT)
                        .border_type(BorderType::QuadrantOutside))
            };

            let [lsb_min_freq_area, _lsb_spacer1, lsb_max_freq_area, _lsb_spacer2, lsb_str_filer_area, _lsb_spacer3, lsb_detect_mode_area, _lsb_spacer4, lsb_vol_steepness_area] = {
                Layout::vertical([
                    Constraint::Length((lsb_min_freq_widget.line_count(left_sidebar_width - 2) + 2) as u16),
                    Constraint::Length(1),
                    Constraint::Length((lsb_max_freq_widget.line_count(left_sidebar_width - 2) + 2) as u16),
                    Constraint::Length(1),
                    Constraint::Length((lsb_str_filter_widget.line_count(left_sidebar_width - 2) + 2) as u16),
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
                    .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                    .title(" Minimum Frequency ")
            };
            let lsb_max_freq_border_block = {
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                    .title(" Maximum Frequency ")
            };
            let lsb_str_filter_border_block = {
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                    .title(" Noise Filter ")
            };
            let lsb_detect_mode_border_block = {
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                    .title(" Detection Mode ")
            };
            let lsb_vol_steepness_border_block = {
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                    .title(" Volume Steepness ")
            };

            let lsb_min_freq_content_area = lsb_min_freq_border_block.inner(lsb_min_freq_area);
            let lsb_max_freq_content_area = lsb_max_freq_border_block.inner(lsb_max_freq_area);
            let lsb_str_filter_content_area = lsb_str_filter_border_block.inner(lsb_str_filer_area);
            let lsb_detect_mode_content_area = lsb_detect_mode_border_block.inner(lsb_detect_mode_area);
            let lsb_vol_steepness_content_area = lsb_vol_steepness_border_block.inner(lsb_vol_steepness_area);

            frame.render_widget(lsb_min_freq_border_block, lsb_min_freq_area);
            frame.render_widget(lsb_max_freq_border_block, lsb_max_freq_area);
            frame.render_widget(lsb_str_filter_border_block, lsb_str_filer_area);
            frame.render_widget(lsb_detect_mode_border_block, lsb_detect_mode_area);
            frame.render_widget(lsb_vol_steepness_border_block, lsb_vol_steepness_area);

            frame.render_widget(lsb_min_freq_widget, lsb_min_freq_content_area);
            frame.render_widget(lsb_max_freq_widget, lsb_max_freq_content_area);
            frame.render_widget(lsb_str_filter_widget, lsb_str_filter_content_area);
            frame.render_widget(lsb_detect_mode_widget, lsb_detect_mode_content_area);
            frame.render_widget(lsb_vol_steepness_widget, lsb_vol_steepness_content_area);

            // Render Volume Display
            let vol_display_block = {
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                    .title(" Volume ")
            };
            let vol_display_widget = {
                Paragraph::new(create_bar(normalize(last_vol, vol_filter.get(), vol_steepness.get()),
                                          (vol_display_width - 4) as usize,
                                          (main_app_height - 2) as usize))
                    .block(Block::default()
                        .borders(Borders::LEFT | Borders::RIGHT)
                        .border_type(BorderType::QuadrantOutside)
                        .border_style(Style::default().fg(Color::Blue).bg(Color::Green)))
            };

            let vol_display_content_area = vol_display_block.inner(vol_display_area);

            frame.render_widget(vol_display_block, vol_display_area);
            frame.render_widget(vol_display_widget, vol_display_content_area);

            // Render Right Sidebar
            if detect_mode == 0 {
                // Sub-Harmonic Summing
                let rsb_num_harmonics_widget = {
                    Paragraph::new(format!("{}", shs_num_harmonics.get()))
                        .style(Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside))
                };
                let rsb_harmonic_decay_widget = {
                    Paragraph::new(format!("{}", shs_num_harmonics.get()))
                        .style(Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside))
                };
                let rsb_comp_turn_point_widget = {
                    Paragraph::new(format!("{}", shs_comp_turn_point.get()))
                        .style(Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside))
                };
                let rsb_favor_low_widget = {
                    Paragraph::new(format!("{}", if shs_favor_low.get() { "Yes" } else { "No" }))
                        .style(Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside))
                };
                let rsb_peak_threshold_widget = {
                    Paragraph::new(format!("{}", shs_peak_threshold.get()))
                        .style(Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside))
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
                        .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                        .title(" Number of Harmonics ")
                };
                let rsb_harmonic_decay_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                        .title(" Harmonic Decay ")
                };
                let rsb_comp_turn_point_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                        .title(" Compression Turning Point ")
                };
                let rsb_favor_low_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                        .title(" Favor Low Frequencies ")
                };
                let rsb_peak_threshold_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                        .title(" Peak Threshold ")
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
                // Two-Way Mismatch
                let rsb_predicted_harms_widget = {
                    Paragraph::new(format!("{}", twm_predicted_harms.get()))
                        .style(Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside))
                };
                let rsb_measured_peaks_widget = {
                    Paragraph::new(format!("{}", twm_measured_peaks.get()))
                        .style(Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside))
                };
                let rsb_freq_penalty_widget = {
                    Paragraph::new(format!("{}", twm_freq_penalty.get()))
                        .style(Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside))
                };
                let rsb_amp_weight_widget = {
                    Paragraph::new(format!("{}", twm_amp_weight.get()))
                        .style(Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside))
                };
                let rsb_freq_weight_widget = {
                    Paragraph::new(format!("{}", twm_freq_weight.get()))
                        .style(Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside))
                };
                let rsb_error_ratio_widget = {
                    Paragraph::new(format!("{}", twm_error_ratio.get()))
                        .style(Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: false })
                        .block(Block::default()
                            .borders(Borders::LEFT | Borders::RIGHT)
                            .border_type(BorderType::QuadrantOutside))
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
                        .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                        .title(" Predicted Harmonics ")
                };
                let rsb_measured_peaks_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                        .title(" Measured Peaks ")
                };
                let rsb_freq_penalty_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                        .title(" Frequency Penalty ")
                };
                let rsb_amp_weight_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                        .title(" Amplitude Weight ")
                };
                let rsb_freq_weight_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                        .title(" Frequency Weight ")
                };
                let rsb_error_ratio_block = {
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default().fg(Color::Blue).bg(Color::Green))
                        .title(" Error Ratio ")
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
        })?;

        thread::sleep(Duration::from_millis(16));

        while event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? && key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Up => {
                        if let Ok(new_min) = min_freq.shift_up_bounded(max_bound) {
                            safe_audio_msg(&out_post, AudioMessage::UpdateMinFreq(new_min))?;
                        }
                    }
                    KeyCode::Down => {
                        if let Ok(new_min) = min_freq.shift_down_bounded(min_pos_freq) {
                            safe_audio_msg(&out_post, AudioMessage::UpdateMinFreq(new_min))?;
                        }
                    }
                    KeyCode::Right => {
                        if let Ok(new_max) = max_freq.shift_up_bounded(max_pos_freq) {
                            safe_audio_msg(&out_post, AudioMessage::UpdateMaxFreq(new_max))?;
                        }
                    }
                    KeyCode::Left => {
                        if let Ok(new_max) = max_freq.shift_down_bounded(min_bound) {
                            safe_audio_msg(&out_post, AudioMessage::UpdateMaxFreq(new_max))?;
                        }
                    }
                    KeyCode::Char('w') => {
                        if let Ok(new_vol_filter) = vol_filter.shift_up() {
                            vol_filter.set(new_vol_filter);
                        }
                    }
                    KeyCode::Char('s') => {
                        if let Ok(new_vol_filter) = vol_filter.shift_down() {
                            vol_filter.set(new_vol_filter);
                        }
                    }
                    KeyCode::Char('r') => {
                        vol_filter.set(vol_filter.default());

                        let new_min = min_freq.default();
                        let new_max = max_freq.default();

                        detect_mode.set(detect_mode.default());

                        safe_audio_msg(&out_post, AudioMessage::UpdateBounds(new_min, new_max))?;
                    }
                    KeyCode::Char('m') => {
                        detect_mode.set(detect_mode.shift_up());
                    }
                    KeyCode::Char('f') => {
                        safe_audio_msg(&out_post, AudioMessage::RequestFreqs)?;
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
        sform => Err(String::from(format!("Unsuported sample format: {sform}")))?
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
        if line.len() != 0 {
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
        for letter in 0usize..12 {
            let new_diff = freq - FREQUENCIES[letter] * 2.0f32.powf(octave as f32);
            if new_diff <= 0.0 {
                let spec_diff = freq - FREQUENCIES[11] * 2.0f32.powf(-1.0);
                if start && (spec_diff < 0.0 || new_diff.abs() > spec_diff.abs()) {
                    return Err((false, String::from("Number too Low to Match")));
                }
                return if new_diff.abs() <= old_diff {
                    Ok((letter, octave))
                } else {
                    Ok((last_let, last_oct))
                }
            }
            old_diff = new_diff;
            last_oct = octave;
            last_let = letter;
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