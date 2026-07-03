# png-export

## Purpose

Export the wallpaper as a deterministic 4K PNG to a predictable location without overwriting existing files.

## Requirements

### Requirement: 4K PNG export

The system SHALL export the wallpaper as a PNG at the selected format's full
resolution (up to 4K). No other output format SHALL be offered.

#### Scenario: Export produces a PNG at format resolution

- **WHEN** the user exports a configuration in the 16:9 desktop format
- **THEN** a PNG file is written with dimensions 3840×2160

#### Scenario: Only PNG is offered

- **WHEN** the user requests an export
- **THEN** the only available output file type is PNG

### Requirement: Deterministic export

The system SHALL produce identical PNG output for identical configurations,
since every generative step is seeded.

#### Scenario: Same configuration yields same image

- **WHEN** the same configuration is exported twice
- **THEN** the two PNG images are pixel-identical

### Requirement: Predictable output location

The system SHALL write exported files to a predictable directory under the user's
Pictures folder, overridable by an environment variable, without overwriting an
existing file.

#### Scenario: File lands in the output directory

- **WHEN** an export completes
- **THEN** the PNG exists in the configured output directory

#### Scenario: Existing file is not overwritten

- **WHEN** an export would collide with an existing file name
- **THEN** the system writes to a new, non-colliding name instead
