## ADDED Requirements

### Requirement: Live preview

The system SHALL provide a live preview that re-renders when the configuration
changes, running the same pipeline as export. The preview SHALL be rasterized at
the physical size it is displayed at, so preview pixels map one-to-one to screen
pixels, and SHALL re-render when the window is resized.

#### Scenario: Preview updates on change

- **WHEN** the user changes a configuration value
- **THEN** the preview re-renders to reflect the new value

#### Scenario: Structural vs style parameters

- **WHEN** a style-only parameter changes (color, stroke width, overlay or grain intensity, smoothing)
- **THEN** the preview re-renders without regenerating the noise field

#### Scenario: Preview matches its displayed size

- **WHEN** the preview area is laid out or resized
- **THEN** the preview is rasterized at the displayed physical size, within performance bounds

### Requirement: Color picker

The system SHALL provide a color picker for every color in the configuration
(background, line color, gradient stops, text color) consisting of labeled R, G
and B sliders with numeric values, a live swatch, and a text field accepting hex
(`#rrggbb`) or `r, g, b` input with a mode toggle between the two notations.
Sliders, swatch and text field SHALL stay in sync in real time in both
directions. Invalid text SHALL be rejected and replaced by the current value.

#### Scenario: Slider movement updates the text live

- **WHEN** the user drags a channel slider
- **THEN** the swatch and the color text field update immediately

#### Scenario: Typed color updates the sliders

- **WHEN** the user types a valid color and confirms with Enter or by leaving the field
- **THEN** the sliders and swatch move to the typed color

#### Scenario: Invalid text is rejected

- **WHEN** the user types text that is not a valid color
- **THEN** the color is unchanged and the field returns to the current value

### Requirement: Text edits apply on focus loss

The system SHALL apply the content of any text field both when the user presses
Enter and when the field loses focus, so no typed value is silently dropped.

#### Scenario: Leaving a field commits it

- **WHEN** the user types into a text field and clicks elsewhere
- **THEN** the typed value is applied as if Enter had been pressed

### Requirement: Editor organization

The system SHALL group the editor controls into titled, collapsible sections
(pattern, colors, lines, effects, text zone, presets) so the user is not exposed
to every option at once. Controls whose effect is disabled by another control
(for example the single line color while a gradient is active) SHALL NOT be
shown. Intensity and normalized position values SHALL be displayed as
percentages from 0 to 100.

#### Scenario: Sections collapse and expand

- **WHEN** the user activates a section header
- **THEN** that section's controls are shown or hidden

#### Scenario: Gradient replaces the single line color

- **WHEN** the line gradient is enabled
- **THEN** the single line color control is not shown

### Requirement: Contextual help

The system SHALL explain non-obvious parameters (such as the seed) through an
info affordance that reveals a short explanation on hover or keyboard focus.

#### Scenario: Seed explanation is available

- **WHEN** the user hovers or focuses the seed info affordance
- **THEN** an explanation of what the seed does is shown

### Requirement: Keyboard navigation and accessibility

The system SHALL make every interactive control reachable and operable by
keyboard (Tab order, Enter or Space to activate, arrow keys to adjust sliders
and combo boxes) with a visible focus indicator, and SHALL expose accessibility
metadata (role, label, value, state) for assistive technologies.

#### Scenario: Controls are keyboard-operable

- **WHEN** the user navigates with Tab and operates a control with Enter, Space or arrow keys
- **THEN** the control activates or adjusts exactly as it would by mouse

#### Scenario: Focus is visible

- **WHEN** a control receives keyboard focus
- **THEN** it shows a visible focus indicator

### Requirement: Export feedback

The system SHALL show a full-window progress overlay while an export runs. On
success the overlay SHALL prominently show the saved file's path as an
activatable link that reveals the file in the platform file manager (selecting
the file on Windows, opening the containing folder elsewhere). On failure the
overlay SHALL show the error. The overlay's card SHALL size to its content
regardless of window proportions.

#### Scenario: Progress is visible during export

- **WHEN** an export is running
- **THEN** a full-window overlay with a progress indicator is shown

#### Scenario: Saved path is revealed on click

- **WHEN** the export succeeds and the user activates the path link
- **THEN** the file is revealed in the platform file manager

#### Scenario: Failure is reported

- **WHEN** the export fails
- **THEN** the overlay shows the error message

### Requirement: Named presets

The system SHALL save and load named presets as TOML, round-tripping the full
configuration so a reloaded preset reproduces the same wallpaper.

#### Scenario: Preset round-trips

- **WHEN** a configuration is saved as a preset and then loaded
- **THEN** the loaded configuration equals the saved configuration

#### Scenario: Preset reproduces output

- **WHEN** a wallpaper is exported, its preset saved, then reloaded and re-exported
- **THEN** both exports are pixel-identical

#### Scenario: Presets predating a field still load

- **WHEN** a preset saved before a configuration field existed is loaded
- **THEN** the missing field takes its default value and the preset loads

### Requirement: Preset storage location

The system SHALL store presets under the platform configuration directory,
overridable by an environment variable.

#### Scenario: Preset persists to disk

- **WHEN** a preset is saved
- **THEN** a TOML file for it exists in the presets directory
