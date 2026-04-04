### Requirement: Buffer entry foreground colors are theme-configurable
The system SHALL apply theme-derived foreground colors to file and directory entries in directory buffers. File entries SHALL use the `BufferFileFg` token. Directory entries SHALL use the `BufferDirectoryFg` token. No hardcoded ANSI color codes SHALL remain for directory entry styling.

#### Scenario: Custom directory entry color
- **WHEN** `init.lua` sets `y.theme.BufferDirectoryFg = '#00ff00'`
- **THEN** directory entries in the buffer render with green foreground

#### Scenario: Custom file entry color
- **WHEN** `init.lua` sets `y.theme.BufferFileFg = '#cccccc'`
- **THEN** file entries in the buffer render with light gray foreground

#### Scenario: Default directory color matches current appearance
- **WHEN** no `BufferDirectoryFg` token is set
- **THEN** directory entries render with light blue foreground (matching current hardcoded `\x1b[94m`)

#### Scenario: Default file color
- **WHEN** no `BufferFileFg` token is set
- **THEN** file entries render with white foreground

### Requirement: Statusline permissions foreground is theme-configurable
The system SHALL apply the `StatusLinePermissionsFg` theme token to file permission text in the statusline.

#### Scenario: Custom permissions color
- **WHEN** `init.lua` sets `y.theme.StatusLinePermissionsFg = '#ffaa00'`
- **THEN** the permissions string in the statusline renders with orange foreground

#### Scenario: Default permissions color
- **WHEN** no `StatusLinePermissionsFg` token is set
- **THEN** permissions text renders with gray foreground

### Requirement: Statusline border background is theme-configurable
The system SHALL apply the `StatusLineBorderBg` theme token to the background of the statusline border area.

#### Scenario: Custom statusline border background
- **WHEN** `init.lua` sets `y.theme.StatusLineBorderBg = '#222222'`
- **THEN** the statusline border area renders with dark gray background

#### Scenario: Default statusline border background
- **WHEN** no `StatusLineBorderBg` token is set
- **THEN** the statusline border background renders as black

### Requirement: Directory window borders are theme-configurable
The system SHALL apply `DirectoryBorderFg` and `DirectoryBorderBg` theme tokens to borders inside directory-type windows (parent, current, and preview panes).

#### Scenario: Custom directory window border colors
- **WHEN** `init.lua` sets `y.theme.DirectoryBorderFg = '#444444'` and `y.theme.DirectoryBorderBg = '#111111'`
- **THEN** the right-side borders between directory panes render with the configured foreground and background

#### Scenario: Default directory window border colors
- **WHEN** no `DirectoryBorderFg` or `DirectoryBorderBg` tokens are set
- **THEN** directory window borders render with black foreground and reset (transparent) background

### Requirement: Split borders are theme-configurable
The system SHALL rename the existing `BorderFg` token to `SplitBorderFg` and add a `SplitBorderBg` token. These tokens SHALL apply to vertical split separator borders. The `yeet-buffer` view SHALL use border colors from `BufferTheme` when rendering `Block` border widgets. No hardcoded `Color::Black` SHALL remain in the buffer border rendering code.

#### Scenario: Custom split border colors
- **WHEN** `init.lua` sets `y.theme.SplitBorderFg = '#555555'` and `y.theme.SplitBorderBg = '#000000'`
- **THEN** the vertical split separator renders with the configured foreground and background

#### Scenario: Default split border colors
- **WHEN** no `SplitBorderFg` or `SplitBorderBg` tokens are set
- **THEN** split borders render with black foreground and reset (transparent) background

#### Scenario: BorderFg token is removed
- **WHEN** `init.lua` references `y.theme.BorderFg`
- **THEN** the token does not exist; users must use `SplitBorderFg` instead
