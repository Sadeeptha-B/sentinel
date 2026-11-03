# SENTINEL
### A Quiet Companion for Courage, Resolve, and Endurance

**Product Brief & Implementation Specification**

**Project:** Sentinel  
**Category:** Minimal desktop companion  
**Primary platforms:** Windows and Linux  
**Suggested stack:** Tauri 2 + React + TypeScript  
**Status:** Greenfield / Minimal Viable Product

---

## 1. Product Vision

Sentinel is a minimalist desktop companion featuring a solitary, battle-worn knight standing directly on the user's desktop.

The knight wears dark, battered steel armor, a tattered navy-blue cloak, and a fully enclosed helmet. He holds a longsword upright before him with both hands.

He is regal, resolute, and unbroken despite the battles he has endured.

He represents **courage, determination, endurance, and the willingness to face whatever challenges arise**.

Unlike a conventional image viewer, Sentinel displays only the knight's figure, without a rectangular background or visible application window.

The knight appears to stand naturally on the desktop, remaining visible alongside other applications.

Sentinel is not a task manager, productivity tracker, or motivational chatbot.

It is a small, persistent visual presence, occasionally offering a few words of encouragement when the user requests them.

**Core proposition: A steadfast knight who stands watch on your desktop, silently embodying determination in the face of adversity.**

---

## 2. Core Experience

When the user launches Sentinel, a knight appears near the edge of their desktop.

There is no application frame, wallpaper, or rectangular image.

Only the knight is visible.

The knight:

- Stands upright, facing forward.
- Wears dark, battle-damaged steel armor with subtle gold detailing.
- Has a fully closed helmet that conceals his face.
- Wears a dark navy-blue, slightly tattered cloak.
- Holds a long sword vertically in front of his body, grasped with both hands.
- Remains still, dignified, and imposing.

The user can reposition and resize the figure.

The knight remains visible above ordinary application windows, without demanding attention.

A simple interaction can reveal a short motivational message.

For example:

> Battered, but unbroken.

Or:

> I stand. I face what comes.

The message appears briefly and fades away.

Most of the time, Sentinel simply stands silently.

---

## 3. Product Principles

### 3.1 Simplicity Above Everything

The application should have a very small feature set.

Do not introduce functionality merely because it is technically possible.

The MVP should consist of:

1. The knight figure.
2. A transparent, always-on-top desktop window.
3. Basic positioning and sizing controls.
4. Optional motivational quotes.
5. Minimal settings and persistence.

### 3.2 Presence Over Interaction

The primary experience is visual.

Users should be able to leave Sentinel running indefinitely without interacting with it.

The knight is not a chatbot or interactive game character.

### 3.3 Motivation Without Pressure

Sentinel represents courage and resolve, but should never become another demanding productivity tool.

There are no notifications, scores, streaks, performance metrics, or productivity reminders.

Motivational phrases appear only when requested.

The app should also acknowledge rest and recovery as legitimate parts of living.

### 3.4 Visual Dignity

The knight should feel like a formidable, battle-tested warrior.

Avoid cartoonish presentation, exaggerated animations, excessive visual effects, or intrusive interface elements.

His stillness is part of his identity.

### 3.5 Low Resource Consumption

Sentinel is designed to run while the user performs other activities.

It should consume minimal CPU and memory when idle.

---

## 4. Visual Design

### 4.1 The Knight

Use the provided knight artwork as the primary visual reference.

The intended appearance is:

- Full-body medieval knight.
- Completely enclosed helmet, with no visible face.
- Dark, battle-scarred plate armor.
- Tarnished gold heraldic decorations, including lion motifs.
- Dark navy-blue cloak with damaged edges.
- A long, straight sword held vertically in front.
- Both gauntleted hands gripping the sword's hilt.
- Strong, upright, symmetrical posture.
- Feet firmly planted.
- An atmosphere of resilience, dignity, and determination.

The knight should appear powerful without needing an aggressive or exaggerated pose.

### 4.2 Background Removal

**The knight must be rendered against a genuinely transparent background.**

The original reference image includes a battlefield. This background must not appear in the application.

Prepare a separate transparent character asset by carefully extracting the knight from the supplied reference image, or use an equivalent character-only asset derived from it.

Requirements:

- Preserve the full knight, including helmet, sword, cloak, and boots.
- Remove the battlefield, sky, scenery, and other characters.
- Preserve fine silhouette details and damaged cloak edges.
- Avoid obvious cutout artifacts or dark halos.
- Use a transparent PNG or WebP with alpha transparency.
- Preserve sufficient resolution for resizing.

The figure should visually stand on the desktop without appearing inside a picture frame.

If accurate extraction is not feasible with the available tools, report the asset limitation rather than substituting a visibly rectangular image.

### 4.3 Window Appearance

The desktop overlay should have:

- No title bar.
- No visible border.
- No background fill.
- No permanent toolbar.
- No visible application chrome.
- A transparent window surface.
- An appropriately sized bounding area around the knight.

The window should not unnecessarily occupy a large invisible rectangle.

Any empty pixels surrounding the figure should be transparent.

### 4.4 Motivational Text

When requested, a motivational phrase may appear in a small, elegant text overlay near the knight.

Suggested visual treatment:

- Restrained serif or similarly dignified typography.
- Soft white or muted gold lettering.
- Subtle dark shadow for readability.
- Gentle fade-in and fade-out.
- No permanent text overlay.
- No bright cards or notification bubbles.

The message should be readable without obscuring the figure.

---

## 5. Core MVP Features

### 5.1 Transparent Desktop Companion

Implement a native transparent desktop window containing the knight figure.

Requirements:

- Genuine transparent background.
- Always-on-top behavior.
- No standard title bar or frame.
- Full-body knight rendered with correct proportions.
- No unnecessary desktop obstruction.
- No application content other than the knight and temporary interaction elements.

This is the central feature of Sentinel.

### 5.2 Positioning and Resizing

The user must be able to position the knight anywhere on the desktop.

Requirements:

- Drag the knight to move the window.
- Resize the knight proportionally.
- Preserve aspect ratio.
- Support multiple monitor configurations where possible.
- Remember the last position and scale.
- Restore the knight to a visible screen location after display changes.

A small default size, approximately 300–450 pixels tall, is appropriate, but the figure should be resizable.

The interaction method should not interfere with the motivational quote controls.

### 5.3 Always-on-Top and Visibility

Sentinel should remain visible above ordinary applications.

Provide a minimal method to:

- Hide the knight.
- Show the knight.
- Toggle always-on-top if desired.
- Exit the application.

A system tray menu is a suitable solution.

If global keyboard shortcuts are supported reliably, a configurable show/hide shortcut may be included.

There must always be a reliable way to restore the knight after hiding it.

### 5.4 Motivational Quotes

Provide a small built-in library of motivational quotes.

The user should be able to request a quote by clicking the knight or selecting an appropriate context-menu action.

Suggested initial quotes:

**Resolve**
- "I stand. I face what comes."
- "Let the challenge come."
- "Courage is action in the presence of fear."

**Endurance**
- "Battered, but unbroken."
- "The road is difficult. I continue."
- "I have endured before. I can endure again."

**Determination**
- "Forward."
- "The work before me is mine to undertake."
- "I will meet this moment."

**Recovery**
- "Even the warrior may rest."
- "Rest is not surrender."

**Victory**
- "Another battle behind me."
- "I have gained ground."

The phrases should be available locally, without network access or AI integration.

The quote library may be selected randomly, preferably avoiding immediate repetition.

Do not display messages automatically at fixed intervals.

### 5.5 Optional Stand Watch Mode

Include a simple symbolic mode if it can be implemented without complicating the application.

**Stand Watch**

The user deliberately activates the mode.

The knight remains in position, with an optional brief message:

> I stand ready.

**Stand Down**

The user deactivates the mode.

A brief message may appear:

> The watch is ended. Rest well.

No timers, task tracking, or productivity recording are required.

This feature is secondary to the primary visual companion.

### 5.6 Minimal Context Menu

Right-clicking the knight should reveal a small menu with actions such as:

- Show Quote
- Stand Watch / Stand Down
- Lock / Unlock Position
- Always on Top
- Hide
- Exit

The application should not require a separate settings dashboard.

A system tray menu should provide a fallback for actions that become inaccessible through the floating figure.

---

## 6. Interaction and Click-Through Behavior

A transparent character overlay presents a particular challenge: preventing accidental interference with ordinary desktop use.

Consider supporting two interaction states.

### Interactive Mode

The user can:

- Drag the knight.
- Resize the figure through an appropriate control or shortcut.
- Open its context menu.
- Request a motivational quote.

### Passive Mode

The knight remains visible but does not interfere with desktop interactions.

Ideally, mouse clicks pass through the window to underlying applications.

This is especially useful if the knight overlaps a working area.

**Important:** Click-through must never make the application impossible to control.

The user must be able to disable click-through through a system tray action or reliable global shortcut.

Platform support should be validated explicitly.

For the MVP, prioritize reliable interaction and recovery over implementing advanced transparency behaviors that are not consistently supported.

---

## 7. Technical Implementation

### Suggested Stack

- **Desktop framework:** Tauri 2
- **Frontend:** React + TypeScript
- **Build tooling:** Vite
- **Styling:** Lightweight CSS
- **Assets:** Transparent PNG or WebP
- **Persistence:** Tauri-supported local application settings
- **Testing:** Vitest for relevant logic, supplemented by desktop integration tests

No backend server, cloud database, authentication, or external API is required.

Avoid introducing complex state-management libraries unless there is a demonstrated need.

### Architecture

Keep the implementation small, with a few clear responsibilities.

**Desktop Window Management**

Handles transparency, always-on-top behavior, dragging, resizing, positioning, visibility, and click-through.

**Character Renderer**

Displays the transparent knight asset, maintaining aspect ratio and visual quality.

**Motivational Quote System**

Selects and displays phrases with lightweight transitions.

**Settings Persistence**

Stores position, scale, always-on-top preferences, and other essential options.

### Platform Considerations

**Windows**

Target standard Windows desktop behavior, including transparent windows and always-on-top positioning.

**Linux**

Account for X11 and Wayland differences.

Transparent windows, always-on-top behavior, pointer passthrough, and global shortcuts may have compositor-specific limitations.

Do not assume all desktop integrations behave identically across platforms.

Clearly document platform restrictions and provide graceful fallback behavior.

---

## 8. Persistence

Remember only the essential settings:

- Window position.
- Character scale.
- Visibility preferences.
- Always-on-top state.
- Interaction / lock mode where appropriate.

Use local storage appropriate to Tauri.

The application should restore its previous configuration when reopened.

Ensure invalid coordinates or monitor changes do not leave the knight inaccessible.

---

## 9. Explicitly Out of Scope

Do not implement the following in the initial version:

- Custom image importing.
- Image galleries or playlists.
- Multiple character selection.
- Character customization.
- Multiple knight skins.
- Atmospheric animation.
- Animated character movements.
- AI-generated motivational content.
- Chat functionality.
- Productivity tracking.
- Task management.
- Focus timers.
- Cloud synchronization.
- User accounts.
- Complex settings dashboards.
- Social features.
- Game mechanics.

These may be considered in future iterations, but none is required to make Sentinel useful.

**The initial version should feature one well-presented knight, not a generalized desktop companion platform.**

---

## 10. Acceptance Criteria

The first version is complete when:

1. Sentinel launches as a real desktop application.
2. The knight is displayed without any visible background or rectangular window.
3. The knight is fully visible, including the sword and feet.
4. The knight remains above ordinary application windows.
5. The user can reposition the figure.
6. The user can resize the figure while preserving its proportions.
7. The application remembers the knight's position and size.
8. The user can request a motivational phrase.
9. The phrase appears elegantly and disappears without requiring interaction.
10. The user can hide, restore, and exit the application reliably.
11. The application remains lightweight during extended idle use.
12. Core functionality works without an internet connection.
13. Desktop-specific limitations are documented accurately.

Click-through and Stand Watch are desirable additions, but should not delay an otherwise complete and polished core implementation.

---

## 11. Implementation Plan

### Phase 1 — The Knight

Establish the Tauri application and prepare the transparent knight asset.

Implement a frameless, transparent window displaying only the figure.

Ensure the image is accurately rendered, correctly proportioned, and visually compelling.

**Deliverable:** A knight standing directly on the desktop.

### Phase 2 — Desktop Companion Behavior

Implement:

- Always-on-top.
- Dragging.
- Proportional resizing.
- Position persistence.
- Hide/show controls.
- System tray integration where supported.

**Deliverable:** A reliable desktop companion that can remain visible during ordinary computer use.

### Phase 3 — Motivational Interactions

Implement:

- Local quote library.
- Click-to-display motivational phrases.
- Subtle text transitions.
- Optional Stand Watch / Stand Down.

**Deliverable:** A visually minimal companion with a small amount of intentional motivational interaction.

### Phase 4 — Polish and Reliability

Improve:

- Transparency and asset edges.
- Desktop positioning.
- Interaction ergonomics.
- Resource consumption.
- Window recovery behavior.
- Multi-monitor handling.
- Platform compatibility.

Optionally implement click-through once a reliable recovery mechanism exists.

---

## 12. Instructions for the Coding Agent

You are tasked with building Sentinel, a minimalist desktop companion featuring a single battle-worn knight.

The primary objective is to create a **beautiful, transparent, always-on-top character overlay**, not a conventional desktop application window.

Use the provided knight artwork as the reference and prepare a high-quality transparent figure asset.

Prioritize native desktop integration, low resource consumption, and polished visual presentation.

Begin with the simplest possible working version:

1. Establish a Tauri application.
2. Prepare and display the transparent knight asset.
3. Implement a frameless, transparent, always-on-top window.
4. Support moving and resizing the figure.
5. Persist the minimal window settings.
6. Add motivational quote interactions.
7. Implement reliable hide/show and exit controls.
8. Test actual desktop behavior where the environment permits.

Avoid unnecessary infrastructure, elaborate component architecture, or features outside the specified scope.

Make reasonable implementation decisions autonomously, but clearly document any platform limitations or asset-processing constraints.

Do not substitute a rectangular image viewer for the transparent character overlay.

Do not introduce customization systems or additional characters.

If the original artwork cannot be accurately converted into a transparent figure, identify that limitation explicitly rather than misrepresenting a poor extraction as complete.

### Final Guiding Principle

Sentinel should feel like a knight standing beside the user, not software competing for their attention.

He does not speak unless invited.

He does not measure progress.

He does not demand success.

He simply stands: armored, battered, and determined.

**A quiet reminder that courage is not the absence of hardship, but the willingness to face it.**