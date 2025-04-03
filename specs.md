# Keyboard-Based Mouse Control Specification

### Introduction

In Linux, user input events (like mouse and keyboard actions) typically come from hardware devices managed by the kernel's **input subsystem**.
When running a display server such as X11, software like `xmodmap` or `xkb` can remap keys at that higher level.
However, going directly to the kernel using the `uinput` (user input) module bypasses X-level abstractions,
allowing the creation of virtual input devices visible system-wide. This is more "low-level" than X-level solutions and typically runs with elevated privileges.

1. **Difference Between X-level and Kernel-level Approaches**

   - **X-level (xmodmap/xkb)**: Remap keys within an X11 environment. Relies on the X server.
   - **`uinput`**: Create a virtual device by writing directly to `/dev/uinput`. The kernel then treats this virtual device like real hardware, enabling input events recognized across all sessions (including non-X).

2. **Enabling the `uinput` Kernel Module**

   - The `uinput` module must be loaded:
     ```bash
     modprobe uinput
     ```
   - If you see a "not found" error, your kernel may need to include or build the `uinput` module.

3. **What We're Doing Here**
   - We want to create a _virtual device_ to issue mouse events (movements, clicks, etc.).
   - We'll open `/dev/uinput`, configure the device with the event types we'll generate, and then write event structs to simulate those actions.
   - We'll need to read events from the real keyboard device to determine when to generate mouse events.

---

### Detailed Specification

Below is a step-by-step outline to implement a low-level mouse control system using `uinput`. Another person (or LLM) can use these steps to develop the final code in Rust or C.

1. **Prepare the Environment**

   - **Ensure `uinput` is available**:
     1. Load `uinput` with `modprobe uinput`.
     2. Confirm `/dev/uinput` (or sometimes `/dev/input/uinput`) is present on your system.
   - **Check Permissions**:
     - The user running the program needs sufficient privileges (often requires `root`, or membership in a group set up to access `uinput` directly).

2. **Open and Monitor the Real Keyboard Device**

   - Identify the keyboard device in `/dev/input/eventX` (use tools like `evtest` to find it).
   - Open the keyboard device in read mode to capture key events.
   - Set up an event loop to read keyboard events and determine when to trigger mouse actions.

3. **Open the UInput Device**

   - Open `/dev/uinput` in read-write mode (e.g., `O_RDWR`).

4. **Configure the Virtual Device**

   - Use the `_IOW` "ioctl" commands to enable the types of events you plan to generate. For example:
     1. `UI_SET_EVBIT` to enable `EV_KEY` (for button clicks) or `EV_REL` (for relative mouse movement).
     2. `UI_SET_KEYBIT` to enable specific mouse button codes, such as `BTN_LEFT` or `BTN_RIGHT`.
     3. `UI_SET_RELBIT` to allow relative movement codes: `REL_X`, `REL_Y`, possibly `REL_WHEEL` etc.
   - Fill out a `uinput_user_dev` structure (or analogous in Rust) specifying **device name** and typical parameters like `absmax` for absolute events if needed (in case you want an absolute pointer).

5. **Create the Virtual Device**

   - Use an `ioctl` call like `UI_DEV_CREATE` to register the device with the kernel.
   - Once created, the device appears in `/dev/input` as a new "inputX" node (depending on your system's naming).

6. **Process Keyboard Events and Inject Mouse Events**

   - When specific keyboard events are detected from the real keyboard:
     - Construct an `input_event` struct (in C) or the Rust equivalent. Examples:
       - For a left-click press event:
         - Type = `EV_KEY`
         - Code = `BTN_LEFT`
         - Value = 1 (pressed)
       - For a left-click release event:
         - Type = `EV_KEY`
         - Code = `BTN_LEFT`
         - Value = 0 (released)
       - For a mouse movement event:
         - Type = `EV_REL`
         - Code = `REL_X` or `REL_Y`
         - Value = <positive or negative integer>
     - Always send a "synchronization" event (`EV_SYN` with `SYN_REPORT`) after each set of events so the kernel updates the state properly.

7. **Close the Virtual Device**

   - Optionally, when you're done, clean up by calling `UI_DEV_DESTROY` (or the Rust equivalent) and closing the handle to `/dev/uinput`.

8. **Security Considerations**
   - Because you're injecting events at the kernel level, your program can produce input events system-wide, including for processes not running under the same user.
   - Make sure only trusted, privileged users can run this software or manipulate `/dev/uinput`.

---

### Requirements

1. **Key Bindings**:

   - Use H, J, K, L for horizontal and vertical mouse movement.
   - Hold Shift to increase movement velocity.
   - Press Shift+S for left-click; Shift+D for right-click.
   - Diagonal movement is supported by pressing two movement keys simultaneously (e.g., h, j).
   - As a consequence a combination of 2-3 keys is possible (3 keys when diagonal movements with modifier).

2. **Movement Behavior**:

   - Base movement speed: 5-10 pixels per event.
   - When Shift is held, movement speed is tripled.
   - Mouse movement stops when keys are released.
   - Continuous movement occurs while keys are held down.
   - If conflicting keys are pressed (e.g., both H and L), events will be emitted anyway and naturally cancel each other.

3. **Configuration**:

   - Use TOML format for configuration.
   - Configurable parameters include speed and keyboard device.
   - Config file location specified via CLI with `--config` parameter.

4. **Device Handling**:

   - Focus on a single keyboard device specified in config.

5. **Error Handling**:

   - Exit with error message if unable to access required devices.

6. **Termination**:
   - No specific key combination to exit the program.
   - Designed to run as a systemd service, terminated externally.
   - On termination, call UI_DEV_DESTROY and deallocate any memory.

### Non-Functional Requirements

1. **Future Enhancement**:
   - Implement acceleration over time when keys are held down.

### Summary

1. Load and confirm `uinput` is ready.
2. Identify and open the real keyboard device to monitor for key events.
3. Open `/dev/uinput` with the correct privileges.
4. Configure the virtual device to handle mouse events (buttons + relative movement).
5. Write `UI_DEV_CREATE` to register the virtual device.
6. Process keyboard events and translate specific keys into mouse actions.
7. Write input events with `EV_KEY`/`EV_REL`, plus a `SYN_REPORT`.
8. Close with `UI_DEV_DESTROY` when done.

Using `uinput` is a powerful approach—unlike X-level tools, it can work under Wayland or without any graphical environment at all—and it can simulate a true hardware device from the software level.
