# Changelog

All notable changes to this project are documented in this file.

## [0.3.2] - 2026-10-13

- Fixed the corner video player showing YouTube's "Error 153" instead of playing: the video is now embedded from a page the app serves on this Mac only (127.0.0.1), since YouTube rejects embeds from the app's own page.

## [0.3.1] - 2026-10-13

- The video player now comes with a default video, so it plays during builds and deploys without setting a link first. A link saved in Settings still takes its place.

## [0.3.0] - 2026-10-13

- Introduced a Liquid Glass design: the window is see-through glass on macOS, with frosted panels, light-catching edges, a pointer-following highlight and capsule buttons across every page, dialog, menu and button. The sidebar floats as a glass panel holding the window controls, and project cards show their glass only on hover. The app's light / dark choice also sets the window's glass.
- Added smooth animations: dialogs and menus spring open and melt away, buttons give a soft spring when pressed, and cards and pages fade in.
- Added a video player: plays a YouTube video in the bottom-right corner while a build or deploy runs, sliding up from the bottom. Set the link, its volume (35% by default) or turn it off in Settings > General.
- Added a light / dark switch to the logs: the sun / moon button next to the log search changes every log panel at once and is remembered. The error dialog now has log search too.
- Run logs are easier to read: the box borders drawn by Flutter's `logger` package and the `flutter:` prefix are removed, each message's time is shown in front of it, and repeated headers are only shown when they change.
- HTTP requests in the run logs are grouped: the URL, headers and body form one block, and when the app logs its request with the response status (`<-- status: 200 GET: <url>`), the response appears right under its request.
- Test logs are easier to read: `flutter test` lines show the file and test name instead of long paths and counters, failing tests are marked in red with their error underneath, and Jest output is no longer shown as warnings.
- Fixed iOS Shortcuts indexing messages (which list every app installed on the device) showing up in run logs.
- Fixed responses showing escaped characters (`\u0131`) instead of Turkish letters in run logs.

## [0.2.15] - 2026-10-13

- Added "Run tests": a new button on each project card runs the project's tests and shows which passed, failed or were skipped, with the failure message under each failing test and the full log one click away. Works with Flutter (`flutter test`), React Native / Expo (Jest), native Android (Gradle unit tests) and native iOS (`xcodebuild test` on an iPhone simulator).

## [0.2.14] - 2026-10-12

- Fixed iOS production deploys failing at the end with "Precheck cannot check In-app purchases with the App Store Connect API Key": the build was uploaded but not submitted for review. Precheck now skips in-app purchases for every project, including existing Fastfiles whose `deliver` call doesn't set it.

## [0.2.13] - 2026-10-05

- Closing the app while a deploy or run is in progress now asks for confirmation first.
- Added search to the logs dialogs: press Cmd+F (macOS) or Ctrl+F (Windows) to search, with highlighted matches, a match counter and Enter / Shift+Enter to jump between them.
- Run logs are no longer cut to the last 50 lines; the full log is kept from start to finish.
- Removed the two-line log preview from the running project card, leaving just the "View all logs" button.
- Fixed the top edge of a running project card being clipped on the home page.
- Lines are now only shown in red when they contain "error" or "failed" as a whole word.

## [0.2.12] - 2026-10-02

- Added a "What's new" dialog: after an update the app shows the release notes for every version since the one you last used, grouped into new, improved and fixed changes.
- Added a "What's new in this version" button to Settings > About to reopen the current version's release notes.

## [0.2.11] - 2026-10-01

- The app now runs and deploys any mobile project, not just Flutter: React Native, Expo, native iOS (Swift) and native Android (Kotlin) projects are supported alongside Flutter.
- Added a project setup wizard: detects the project kind, runs `expo prebuild` for Expo apps (asking for a missing bundle ID / package name), generates the fastlane `Fastfile`/`Appfile` with the `beta`, `release` and `fetch_locales` lanes, and checks store readiness (ASC key, Team ID, Play service account, Android release signing, app existence in each store).
- Existing fastlane setups are completed in place: only the missing lanes and Appfile settings are added, and a Fastfile that no longer parses is restored. Setup steps can be skipped; a deploy reopens the wizard while something it needs is still missing.
- Explained the manual first store release when the app isn't created in a store yet.
- Non-Flutter deploys bump the version in the native projects (and `app.json` for Expo); native single-platform projects only offer their own store.
- Build & Run works for React Native / Expo on the chosen iOS/Android device, and Stop now terminates the whole run (Metro included).
- Release notes are only deployed for the app's actual store languages; leftover locale folders from other apps no longer add new store languages, and a production deploy stops early if a store's language list can't be fetched.
- Store check errors show fastlane's actual cause instead of "fastlane finished with errors".

## [0.2.10] - 2026-09-22

- Redesigned the sidebar: nav buttons now use a classic borderless style, and the pub.dev/Play Store/App Store promo links became an auto-rotating carousel with real brand icons and gradient colors.
- Moved the Serenay Yazılım logo out of the sidebar into Settings > About, now linking to serenayyazilim.com.
- Replaced the project grid's name/version sort toggle with a single shadcn-style dropdown covering all four sort combinations (name/version × ascending/descending).
- Reworked the project grid header: normalized button/input heights to the standard size, widened the search bar, and moved the sort dropdown and project count above the grid, right-aligned.

## [0.2.9] - 2026-09-10

- Added a Production/Test deploy target toggle: Test uploads to TestFlight (iOS) and Internal Testing (Android) via the fastlane `beta` lane instead of `release`.
- Added sorting to the project grid by name or version, ascending or descending.
- Added a live log viewer to the deploy flow ("view all logs"), matching the build flow's logs dialog.

## [0.2.8] - 2026-09-09

- Redesigned event date/time selection with separate date and time inputs, a live duration summary, and validation that event duration is between 15 minutes and 31 days.
- Enforced App Store's required image dimensions (16:9 for the Card Image, 9:16 for the Detail Page Image) before an upload is accepted, with the requirement shown directly on each image slot.
- Added drag-and-drop image upload for event screenshots, alongside the existing file picker.
- Added a clearer uploading state (overlay with spinner and label) and detection of stuck/failed uploads with options to check status or discard and retry.
- Fixed event screenshot uploads failing with `'sourceFileChecksum' is not an attribute on the resource 'appEventScreenshots'`.
- Event schedule (dates, territories), Deep Link, and the In-App Purchase requirement can now be edited after an event is created, not just at creation time.
- Changed the In-App Purchase requirement field from free text to the two choices App Store Connect actually expects (In-App Purchase Required / Not Required).
- Submit for Review now shows exactly what's missing (Deep Link, In-App Purchase requirement, dates not in the future) before you submit, and surfaces App Store Connect's detailed per-field errors instead of just "This resource cannot be reviewed".
- Added event card thumbnails to the In-App Events list.
- Ignored stray macOS AppleDouble sidecar files (`._*`) repo-wide so they can't be committed by accident.

## [0.2.7] - 2026-09-08

- Added an animated first-run product tour that spotlights the sidebar and first project card, with a fake cursor and click animation; replayable from Settings > About.

## [0.2.6] - 2026-09-08

- Added a dedicated In-App Events screen: browse and search all App Store Connect apps from the sidebar, then manage an app's in-app events in a dialog.
- Moved the project/app count next to the search bar with a larger font on the home and In-App Events screens.
- Home and In-App Events screens now navigate instantly; each shows its own loading state while data loads instead of blocking the whole app.
- Only the project/app list scrolls on the home and In-App Events screens; the search bar and header controls stay fixed.
- Fixed project icon lookup picking up macOS AppleDouble sidecar files.
- Fixed an infinite effect loop in the project settings Assets tab.

## [0.2.5] - 2026-09-07

- Added in-app update checking under Settings > About, backed by tauri-plugin-updater and GitHub Releases.

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
