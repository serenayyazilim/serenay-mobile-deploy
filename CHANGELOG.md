# Changelog

All notable changes to this project are documented in this file.

## [0.2.4] - 2026-09-03

- Unified project settings tabs on a consistent pending-change + save pattern.
- Removed the project card background tint in dark mode.
- Added a splash background color setting; running-project card now auto-closes.
- Refactored the create-project dialog into a multi-step module and enabled the asset protocol.

## [0.2.3] - 2026-08-19

- Fixed the release workflow to build the Intel target on the arm64 runner.
- Fixed the quarantine removal command in the README to use the actual app name.

## [0.2.2] - 2026-08-19

- Renamed app display name to "Serenay Mobile Deploy".
- Made the +1 version bump on deploy optional.
- Show app version number in the sidebar.
- Added Slack webhook configuration to Settings.
- Added issue and pull request templates.
- Documented Homebrew install and first-launch Gatekeeper workaround; added README banner and badges, platform support table.

## [0.2.1] - 2026-08-18

- Updated app icon (purple/blue gradient) with a proper macOS squircle mask.
- Fixed app icon rendering as a plain square on macOS.

## [0.2.0] - 2026-08-18

- Added Assets and Version tabs to the project settings dialog.
- Added hot reload/restart/stop controls and a build logs dialog for `flutter run`.

## [0.1.0] - 2026-08-13

- Initial release: multi-platform deploy panel (App Store Connect, Google Play, AppGallery), Fastlane integration, Firebase integration, English/Turkish UI, Settings dialog with Firebase/ASC/Workspace configuration.
