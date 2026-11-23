# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]

- Token manager: Settings has an MCP tab to create tokens, switch their permissions on and off, give a new secret and delete (show/copy the secret too). The same screen opens in a browser on the local network from the MCP tool `token_manager_link` (new `admin` permission; the link is valid 15 minutes and a new one replaces it). The browser page asks for the main password, `MCP_MAIN_PASSWORD` in `.env` next to `mcp-tokens.json` (made with a random value the first time, shown in Settings, MCP; 5 wrong passwords close the link). Settings, MCP also has an on/off switch for the whole MCP server (file `mcp-off`; off = every connection is refused, tokens are kept). The last token, and the last one with `admin`, cannot be deleted or lose it. Existing tokens do not have `admin` yet: switch it on in the app.

- App Store Connect is reached with the API key only; there is no Apple ID sign-in and no 2-step code any more. The 2FA code dialog, its commands and the prompt detection are removed, and the `fetch_all_versions` / `fetch_locales` lanes and the generated `sermobile_api_key` lane stop with "API key is not set" instead of falling back to an Apple ID. Fastlane runs no longer get a stdin.
- `fetch_all_versions` prints `IOS_DATES=` (release date, else creation date) so `iosReleasedAt` is the App Store date; when the lane gives none the date is left empty instead of the sync time.
- A deploy no longer wipes `android`, `ios`, `syncedAt` and the release dates in `version.json` (it rewrote the file with only `version`).
- `sync_store_versions` returns the sync's `⚠️` lines as `warnings`, so a project that got no store version shows why.

## [0.3.31] - 2026-11-20

- MCP access by permission. Every request now needs a bearer token, also from this Mac, and the permissions switched on for the token decide which tools it may use: `read`, `run` (app and tests), `deploy`, `store` (App Store Connect changes), `project` (version, config, colors, setup) and `system` (git branch, workspace, app update, new Firebase/Sentry projects). Tokens and their permissions are in `mcp-tokens.json`, read on every request, so a permission is switched on or off by editing the list, with no restart; the old `mcp-token` becomes a token named `default` with every permission on the first start. A tool without the permission is left out of `tools/list` and refused with the permission it needs; a tool nobody classified needs `system` (a test checks that every tool is classified). **Claude Code on this Mac must send the token now**: `claude mcp add --transport http --scope user serenay-deploy http://127.0.0.1:47321/mcp --header "Authorization: Bearer <token>"`.

## [0.3.30] - 2026-11-19

- MCP `sync_store_versions` for all projects now runs in the background and returns at once. It outlasts the MCP client's timeout, and a call that timed out used to kill the sync before it saved anything (all `syncedAt` stayed null). `list_projects` returns `lastSync` (`running`, `done` or `failed` with the error) to follow it.
- MCP `sync_store_versions` with `project` is now quick on the App Store side too: the iOS lane only looks up that app (`SYNC_PROJECT`) instead of every app.

## [0.3.29] - 2026-11-19

- Deploy history: every deploy appends one entry per store (date and time, app, version, `apple` or `google_play`, track, `success` or `failed`) to `sermobileboss_deploy_history.json` in the workspace, also when the build fails or the deploy is stopped. The MCP `deploy_history` tool reads it (filter by `project`, `limit`).
- MCP `list_projects` returns `syncedAt`, when the project's store versions were last synced. The sync writes it to each project's `version.json`, also when the versions did not change.
- MCP `list_projects` returns `androidReleasedAt` and `iosReleasedAt`. The iOS date comes from App Store Connect (the version's release date, else its creation date; the `fetch_all_versions` lane prints it as `IOS_DATES=`). Google Play gives no release date, so it is the day a sync first saw that version: versions already live before the first sync show that day.
- MCP `sync_store_versions` takes an optional `project` to sync one app. The App Store lookup still reads every app, so only the Google Play side gets quicker.

## [0.3.28] - 2026-11-17

- Live view, iOS: typing now follows the Mac's keyboard layout. When the simulator's pasteboard is not available, the app reads the active layout (`UCKeyTranslate`) and presses the keys (with Shift or AltGr) that produce each character, so `@`, `$`, `{`, `<`, `"` and the Turkish letters come out right on a Turkish keyboard instead of coming out as `'`, `ç` and so on. With a US layout, or for a character the layout cannot reach, it falls back to `idb ui text`. The layout is read once per run, so restart the run after changing the Mac's keyboard.

## [0.3.27] - 2026-11-17

- Live view, iOS: text is typed by putting it on the simulator's pasteboard (`simctl pbcopy`) and pressing Cmd+V, so it no longer depends on the keyboard layout and keeps Turkish characters. If the pasteboard does not hold the text afterwards (some simulator runtimes ignore `pbcopy`), it falls back to `idb ui text`, which follows the simulator's keyboard layout. A companion or idb client that died is started again on the next request.
- Live view, Android: non-ASCII text (e.g. Turkish) is typed through ADB Keyboard (`com.android.adbkeyboard`) when it is the active keyboard; otherwise the viewer shows how to set it up. ASCII still uses `adb shell input text`.

## [0.3.26] - 2026-11-17

- The live view of the simulator is much faster on iOS: the picture now comes from `idb video-stream` (MJPEG, frames about 500 px wide) instead of one `simctl` screenshot at a time, so it moves at roughly 13 frames per second while the screen is moving instead of about 2. Taps, swipes, text and keys go through one idb client kept open for the session (about 90 ms each instead of 0.5 to 0.9 s for a new `idb` process per tap). Both run on an `idb_companion` the app starts per session on a private unix socket (not reachable from the network) and stops at `stop_app`. If the stream does not start, the old screenshot loop takes over.
- Android now streams with `adb screenrecord` decoded to MJPEG by `ffmpeg` (both optional, the `screencap` loop is the fallback). Android taps and swipes still use `adb shell input`, which is slow to start on some emulators (0.5 to 1.5 s measured on an arm64 emulator).

## [0.3.25] - 2026-11-17

- MCP `run_app` now also returns a link (`http://<Mac IP>:47321/sim/<token>`) to a live view of the running simulator or emulator. Opened in a browser on the same network it shows the screen and passes taps, swipes, typed text and Back/Home/Enter/⌫ keys to it, plus hot reload and hot restart buttons; nothing is installed on the viewing device. Android uses `adb` (screencap, `input`), iOS uses `simctl` and `idb` (found on PATH or in `~/.local/bin`). Android's `input text` types ASCII only.
- The link carries its own random session token, separate from the MCP bearer token, works only from the local network and stops working (404) at `stop_app`. It is also in `status` under `run.viewUrl` and from the new `sim_link` tool.

## [0.3.24] - 2026-11-17

- The app checks for a newer version of itself at start and then once a day while it is open. The result is part of the MCP `status` (under `app`), and new MCP tools `app_version`, `app_check_update` and `app_update` read the app's own version, check now, and install the update and restart (refused while a deploy, app run or test is running).
- The MCP `deploy` tool has a new `allow_untranslated` option (default true): if the release notes can't be translated for some store languages, the deploy continues with the notes as written instead of waiting for the confirmation dialog in the app, which an MCP client could not answer.

## [0.3.23] - 2026-11-16

- A running deploy can be stopped: a "Stop deploy" button on the project card and in the deploy logs, and a new MCP `stop_deploy` tool. It ends ruby, fastlane, xcodebuild and gradle together (and answers a waiting 2FA or translation question with "no"). A stopped deploy is not reported as a failure. Nothing is rolled back; a build already uploaded stays in the store.
- The release notes translation retries up to three times before a store language counts as failed, since the free translation endpoint now and then answers with an error or a rate limit.

## [0.3.22] - 2026-11-16

- The MCP server has new `maestro_flows` and `maestro_flow_write` tools to read and edit the workspace's Maestro E2E flows in `.maestro/`.
- The MCP server has a new `remote_access` tool: this Mac's IP and whether Screen Sharing (VNC) is on, with the steps to use the simulators and emulators by hand from another computer on the same network. Screen Sharing gives access to the whole screen, not only the simulators.

## [0.3.21] - 2026-11-16

- The store version sync shows why App Store versions could not be fetched (e.g. Apple's "A required agreement is missing or has expired") as an error, instead of quietly logging "0 apps found".

## [0.3.20] - 2026-11-13

- The MCP server has a new `test_screenshots` tool: it returns the newest screenshots Maestro saved in `build/maestro` during the last E2E/smoke run (e.g. the screen a flow failed on) as images, with their file paths.

## [0.3.19] - 2026-11-12

- MCP `list_projects` compares the Google Play and App Store versions by x.y.z only and returns one `storeVersion` (x.y.z) when they match. The App Store build number the sync saves is derived from x.y.z, so it never matched Android's real version code and nearly every project showed as different.

## [0.3.18] - 2026-11-12

- "Sync Store Versions" now gets the App Store versions too. It ran fastlane through `bundle exec`, which fails when the Mac's bundler is broken, and it didn't pass the App Store Connect API key, so fastlane fell back to an Apple ID login that asks for a 6-digit code. It now runs `fastlane` the way deploys do and signs in with the workspace's API key, so no code is needed.
- The MCP server has a new `sync_store_versions` tool: it fetches the latest Google Play and App Store versions of every project, saves them and returns the updated project list. It stops with a message if Apple still asks for a 2-step code.
- MCP `list_projects` returns one `storeVersion` when Google Play and the App Store have the same version, and `androidStoreVersion` and `iosStoreVersion` only when they differ.

## [0.3.17] - 2026-11-11

- Fixed "Sync Store Versions" failing with `"\xC3" on US-ASCII` when the app started Ruby without a UTF-8 locale; the script and the app now force UTF-8.
- Google Play and App Store versions are now kept separately, since they can differ. The sync writes `android` and `ios` next to `version` in each project's `version.json`. The project settings version tab and the sync result show both, and the MCP `list_projects` tool returns `androidStoreVersion` and `iosStoreVersion`.

## [0.3.16] - 2026-11-10

- A deploy no longer waits forever when Apple rejects the uploaded iOS build. Apple reports rejections like ITMS-90683 (a missing Info.plist purpose string) only by email, and the build never shows up in App Store Connect. The deploy now stops after 30 minutes of waiting for it to show up and says to check the email. A build that does show up is waited for until it's processed. Set `BUILD_WAIT_MINUTES` to change the limit.

## [0.3.15] - 2026-11-10

- When less than 20 GB of disk space is free, a deploy first deletes caches that builds recreate: Xcode DerivedData, Xcode archives older than 30 days, and the Gradle caches and wrappers of Gradle versions the project doesn't use. The current Gradle version and the downloaded Android libraries are kept. The deploy log shows how much space was freed; set `MIN_FREE_GB` to change the limit.

## [0.3.14] - 2026-11-10

- The deploy dialog has a "Smoke and Maestro tests" box, off by default. Ticked, the deploy runs the project's own `smoke/android.sh` or `smoke/ios.sh` after the build and before the upload, and stops if it fails. Projects without these scripts skip it and say so in the log. The MCP `deploy` tool takes the same option as `smoke_test`.

## [0.3.13] - 2026-11-06

- The sidebar shows the workspace's git branch under its name and switches it: pick a branch and the app fetches, puts uncommitted changes aside in a git stash (they aren't deleted; `git stash pop` brings them back), switches, pulls the latest commits and runs `flutter pub get`. It shows how many commits the branch is behind, and waits while a deploy or run is going on.
- The MCP server can do the same with the new `git_status` and `git_switch` tools, e.g. "switch sermobilepro to main and update it".

## [0.3.12] - 2026-11-06

- The MCP server has a new `logs` tool: every log line of the deploy, the app run or the tests, not just the last 30, so an AI assistant can find why a deploy failed. It keeps the last failed deploy's logs after the app clears them, can show only matching lines (e.g. `error|failed`) or the last N, and can read log files under `~/Library/Logs`, such as the gym log with the full xcodebuild output of a failed iOS build.

## [0.3.11] - 2026-11-06

- The MCP server can now be used from other devices on the same network. Requests from them need the token in `~/Library/Application Support/com.serenaymobiledeploy.app/mcp-token` (made on first start) as `Authorization: Bearer <token>`; this Mac still needs none. See the README for the `claude mcp add` command.

## [0.3.10] - 2026-11-02

- A production deploy no longer publishes untranslated release notes without telling you: if a language can't be translated, the deploy asks before the build starts, listing those languages. Continue tries the translation once more and sends your text as written for the languages that still fail; Cancel stops the deploy.
- Unanswered Age Rating questions no longer stop an iOS deploy before the build. The deploy carries on and shows a warning on the project card and in its logs: the build still uploads, and if the submission fails you can answer the questions and submit it from App Store Connect. Fields that aren't required, like the Korean GRAC number, no longer count as unanswered.

## [0.3.9] - 2026-10-30

- serMobilePro workspaces skip old versions again: an app below the minimum version (19.0.0 by default) jumps to it on its next deploy with a version bump. Change it or leave it empty to turn it off in Settings > Workspace Settings.
- serMobilePro apps get their build number from the version again (19.0.1 gives 190001000). Other projects still go up by 1 on each deploy.

## [0.3.8] - 2026-10-30

- iOS deploys now check App Store Connect before the build starts, instead of failing after it: an agreement that isn't accepted or has expired stops any iOS deploy, and for a production deploy so does a version still waiting for review, in review or pending release, or unanswered Age Rating questions. The message says what to fix and where.

## [0.3.7] - 2026-10-27

- Added an MCP server, so an AI assistant like Claude Code can use the app: deploy, run on a device, hot reload, run tests, switch workspaces, change the version, settings, colors and assets, check and fix the store setup, manage App Store In-App Events, and create Firebase and Sentry projects. Deploys, runs and tests show in the app exactly as if their button was pressed. See "AI control (MCP)" in the README.
- The build number now goes up by 1 on each deploy instead of being computed from the version (19.0.3 used to give 190003000). It continues from the current number, so the stores still accept it.

## [0.3.6] - 2026-10-22

- Fixed saving the Apple Team ID failing with "Not a directory (os error 20)" in projects copied from an external disk: the hidden "._" files macOS leaves there were taken for the Xcode project. They are skipped now.

## [0.3.5] - 2026-10-21

- Fixed the API URL and Contact URL losing their "www." when a project's settings were saved: the field now shows the address as it is in the project (only https:// and /sermobileboss/ are hidden), and saving or building no longer changes it. Projects that already lost it need "www." typed back in once.

## [0.3.4] - 2026-10-15

- Fixed deploys stopping with "Apple 2-step verification required" and the 6-digit code window opening without a code ever arriving: a line in fastlane's update notes that mentions two-factor authentication was taken for Apple asking for a code. Only Apple's real code prompt opens the window now, and fastlane's update notes are no longer printed.
- Run tests now switches a multi-app workspace to the project you clicked before testing, the same way Start does, so the tests run against that project instead of whichever was active.
- The corner video player can be dragged anywhere on screen: hover over it and a handle appears at the top. It stays inside the window.
- The In-App Events list is now a two-column grid of cards: each event's image is shown large, with its name, badge and dates underneath and its status on the image.

## [0.3.3] - 2026-10-14

- Stop now stops a run right away: pressing it while the project is still being prepared or built ends the run on the spot, instead of waiting for the app to start on the device first.
- A deploy now ends with a macOS notification saying whether it succeeded or failed.
- A progress bar on the app's Dock icon shows how far a running deploy, build or test has got, and turns red if it fails.
- The app icon now fills its rounded shape on macOS 26 and later, like other Mac apps, instead of showing up small.
- Fixed deploy warning that a project had no splash image when its splash color had been changed in the app: the image in the project's Launch folder is now used.

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
