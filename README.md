# RustLocalPitch (v0.1.0-alpha)
> A real-time audio pitch detection app, with a Terminal User Interface (TUI) (and Graphical User Interface (GUI) coming soon) using the same custom backend library, all written in Rust.

## Features
- **Real-time audio pitch tracking**
- **Completely Local and Offline**
- **Multiple Pitch Estimation Algorithms:**
    - **Sub-Harmonic Summing (SHS):** Efficient algorithm, the best all-rounder
    - **Peak Picking:** Extremely lightweight, less accurate but sufficient for some input types
    - **Two-Way Mismatch (TWM):** High-precision spectral matching, more intensive but can be more accurate through noise
> ⚠️ **v0.1.0-alpha Release Note on TWM:** The Two-Way Mismatch algorithm is currently untable but is included for preview and testing purposes.

[Demo Video for RustLocalPitch](https://github.com/user-attachments/assets/a5f37aca-ced6-4bfc-99fe-903257f779e0)

## Requirements
- **TrueColor Support:** Must be used in a TrueColor (24-bit) compatible terminal
  - Most modern terminals support it, including the Windows Terminal, iTerm2, Alacritty, & Kitty
  - You can check whether yours does [here](https://github.com/termstandard/colors#truecolor-support-in-output-devices)
- **Audio Input:** Your device must have an audio input connected to it (or the app will just crash), and the input you want to use must be set at the default device (the one set in settings).

## Installation
- The application is distributed as a portable application, so you can just download the executable (`.exe`) file and run it!
- However, you will need to know your CPU architecture to pick the right version:
  - For Windows:
    1. Open the Command Prompt (press the Windows Key and type 'cmd', it should pop up)
    2. Type `echo %PROCESSOR_ARCHITECTURE%` and hit Enter
       - If you see `AMD64` or `x64`, go with `RustLocalPitch-v0.1.0a-x86_64-win.exe`
       - If you see `ARM64`, go with `RustLocalPitch-v0.1.0a-arm64-win.exe`
       - If you see `x86`, sorry, but you're out of luck because this software only works on 64-bit systems.
  - For Linux:
    1. Open the terminal
    2. Type `uname -m` and hit Enter
       - If you see `x86_64`, remember `x86_64` and move on to the next step
       - If you see `aarch64` or `arm64`, remember `arm64` and move on to the next step
       - If you see `i686` or `i386`, sorry, but you're out of luck because this software only works on 64-bit systems.
    3. For Linux, there is a GNU and MUSL version for each architecture:
       - The differences are that GNU is compatible with most modern Linux distros, and are slightly smaller and more performant, while MUSL is marginally larger and slightly less performant.
       - Either way, it shouldn't impact your experience too much, so I recommend downloading `RustLocalPitch-v0.1.0a-x86_64-gnu` or `RustLocalPitch-v0.1.0a-arm64-gnu` and trying it. If it fails with an error mentioning 'missing glib' or something similar, then come back and download `RustLocalPitch-v0.1.0a-x86_64-musl` or `RustLocalPitch-v0.1.0a-arm64-musl` and see if that works.
       - If neither one works, please open a GitHub issue with your specific linux distro, version, and build, plus the version of RustLocalPitch you are using, and I will try to fix it.
    4. Finally, to actually be able to run the file, you might have to enable execution for it:
       - Copy the filepath of the file
       - then go into the console and type `chmod +x ` and then the filepath you copied, then hit Enter.
- The release should include a version of this README corresponding to that version, so you can always reference that even if the main README has been updated.
- Also, feel free to compile it yourself if you have the means

## Usage
- Run the executable directly from the unzipped folder by double-clicking it
- Run the executable in the terminal by:
  - Copying the filepath (including the file).
  - Opening a terminal.
  - Pasting the copied path in and clicking enter.
### Keybinds
- `A`/`D` - Move Selection Left & Right Between Columns
- `W`/`S` - Move Selection Up & Down a Column
- `Arrow Up`/`Arrow Down` - Modify Parameters
- `Scroll Wheel` (on some platforms) - Quickly Modify Parameters
- `R` - Reset Currently Selected Parameter
- `P` - Reset All Parameters
- `Backspace` - Toggle Hide Selection (Also Reappears on `W`/`A`/`S`/`D`)
- `Q` - Quit the Application Cleanly (Returns Terminal to Previous State)
### Tooltips
- Tooltip box below main app when any item is selected
- The Left Box shows the selected item's:
  - Name
  - Keys to Modify or `Not Modifiable`
  - Range of the parameter
    - Only if the selected item is modifiable
  - Suggested Range of the parameter if it has one
    - Only if the selected item is modifiable and has one
  - Status of the parameter (`AT MAX`, `ABOVE SUGGESTED RANGE`, `WITHIN SUGGESTED RANGE`, `BELOW SUGGESTED RANGE`, `AT MIN`)
    - Some parameters are Yes/No, in which case the Status will show `SUGGESTED VALUE` or `NOT SUGGESTED VALUE`
- The Right Box shows the selected item's:
  - Description
  - Effect of Increasing It (If applicable)
  - Effect of Decreasing It (If applicable)
  - Explanation of Current Choice (If applicable)
- A Note on Ranges:
  - A range beginning with `[` means the start number is inclusive, if it starts with `(` the start is exclusive
  - A range ending with `]` means the end number is inclusive, if it ends with `)` the end is exclusive

### Known Issues (v0.1.0-alpha Release)
- Two-Way Mismatch completely broken and unusable
  - Workaround: Use Sub-Harmonic Summing, Similarly Accurate and Performant

### Roadmap
> I started this project to learn Rust, and so while I am planning to finish the GUI but development will probably not continue after that, unless there is real interest in it. However, this roadmap is here if I do continue it.
- Fix the TWM algorithm
- Standalone GUI frontend using the same core engine
- Data record and export (csv, image, video)
- Audio file processing (+ batch processing)
- Parameter profile saving and loading