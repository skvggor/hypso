## ADDED Requirements

### Requirement: Live preview

The system SHALL provide a live preview that re-renders when the configuration
changes, running the same pipeline as export at a reduced raster size for
responsiveness.

#### Scenario: Preview updates on change

- **WHEN** the user changes a configuration value
- **THEN** the preview re-renders to reflect the new value

#### Scenario: Structural vs style parameters

- **WHEN** a style-only parameter changes (color, stroke width, overlay or grain intensity, smoothing)
- **THEN** the preview re-renders without regenerating the noise field

### Requirement: Named presets

The system SHALL save and load named presets as TOML, round-tripping the full
configuration so a reloaded preset reproduces the same wallpaper.

#### Scenario: Preset round-trips

- **WHEN** a configuration is saved as a preset and then loaded
- **THEN** the loaded configuration equals the saved configuration

#### Scenario: Preset reproduces output

- **WHEN** a wallpaper is exported, its preset saved, then reloaded and re-exported
- **THEN** both exports are pixel-identical

### Requirement: Preset storage location

The system SHALL store presets under the platform configuration directory,
overridable by an environment variable.

#### Scenario: Preset persists to disk

- **WHEN** a preset is saved
- **THEN** a TOML file for it exists in the presets directory
