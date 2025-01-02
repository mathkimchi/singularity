# Devlog

Things I want to log for my own remembering purposes.

## Initial Brainstorm

Inspirations (Things too look at for reference):
- Emacs and Vim for flexibility with subapps and extensions
- Tree-based organization
  - Sidebery Extension
    - Tree based browser tab organization, but slightly different from how I want it.
    - Nodes are tabs, but I want nodes to be more flexible (eg if on a page with sections, those sections would show up in the tree heirarchy) and nodes could have different relationships (not all nodes has its own window or ...)
  - The structure I want resembles processes more than the tab managers, where children nodes might be part of the parent processes
- Display standards
  - Direct pixel buffers
    - This is how apps tell OS's how to display
    - [VGA Terminals](https://en.wikipedia.org/wiki/VGA_text_mode) work similarly, with 2 bytes per character, one byte for attributes (blink, 8 bg, 16 fg) and another byte for character (i think ascii)
    - Definitely do not want this
  - Html
    - Though I usually despise html, I am leaning towards a protocol closest to html right now.
    - The idea of window elements seems like it fits what I need the most.
    - I could have a finite list of allowed window elements
      - Most (if not all) would be rectangular
      - Eg: Plain text, div, button, ...
      - Though practical, I feel a philosophical distaste for this
  - Markdown
    - I like the minimalism of md for documents, but I need much more features for apps
- Rust Cargo and Nix for package management

Use Cases:
- Coding
  - Subapps:
    - Editor
    - File manager
    - Terminal
    - Browser
    - Task organizer
    - Brainstormer
    - Live viewer (for markup languages)
- Note Taking
  - Need to quickly write and organize and read notes
  - Idk, look into org mode and org roam
- General Organization
  - Calendar 
  - Tasks
  - Email
  - Time
- Music making
  - (feels like a whole can of worms I might not want to deal with)

TO REVIEW:
- Tagging
  - Archive
- Environments
  - Divide by project?
- How to deal with multiple screens?
- flexibility+accessibility
  - all core features and most features should be possible with:
    - only keyboard
    - terminal only ui
    - any os
    - audio only (idk, this is a stretch)
- Sync
  - Majority of syncing can be done through git and gh
- How to cope with HTML?
  - Even if this project gets a lot of users, and I create a display standard that can be a HTML replacement for web, I will need to access webpages that use HTML and will not directly support my heirarchy system.
  - I could just ignore that
  - I could give the user an easy way to manually split the html page to sections
    - With something like inspect element
    - Bloat might interfere with this
  - I could give custom extensions for famous webpages based off their api, or converting their html into something else
  - I could just directly support html
    - Probably will break on most complex websites
- Scripting language
  - Rust is a compiled language, how can I support runtime scripts and extensions
- Figure out how the following categories are different and their relations to each other
  - Global Data
  - Subapp Global Data
  - Subapp Instance Data
  - Frontend Data
  - Node
  - UI Element
  - App Instance
  - Environment
  - Window
  - Subapp
  - Tag
  - Project
- Reference Nodes
  - Analogous to symlinks
  - If multiple projects require the calendar, it would be beneficial to have multiple references to one instance calander app.
- Automatic Tree Organizer
  - Minimizes the total distance * frequency of switch of all pairs of two nodes
  - I probably wouldn't use this, but it could be fun
  - Speaking of this, what about cool telemetry (stored locally, opt in) for fun like those graphs in obsidian or smth
- Element
  - Elements are displayables that subapps can use to avoid rewriting code.
  - For example, the textbox element and tree view element
  - Having this also makes it easier to standardize subapps

Maybe the thing that I am looking for is a project management app.
One of the things I want to change is that I don't like the way apps are organized.
It feels ineffecient for some reason. The current organization is divided by:
desktops, then windows (where a window is a single app), then the app's own organization system.
If I am coding, I usually have multiple apps, at least an editor and a browser for a single project.
That means I have to constantly switch between multiple UIs and multiple windows.
When I close and open the project, I then have to reopen all the related things in two different systems.
Is this a really specific minor inconvenience? Yes, and I want to fix that.
I want singularity to set a system where all these different subapps can be organized within the same system.
A good rule of thumb for effecient organization is that things that I switch between often should be organized "close" to each other.

Ultimately apps are just friendly ways to display and modify data.
Its pretty obvious, but I just wanted to say it.
Every project will have corresponding data (not necessarily centralized)

Short-term Development Plan:
- I am just going to start by focusing on the bare minimum coding environment for a single project at a time
- Stick with TUI for now
  - Give subapps buffers from the ratatui library
    - Each cell or character has the following attributes:
      - symbol (the actual character), fg color, bg color, underline color, modifiers (which is a whole bunch of new stuff, go to ratatui::style::Modifier for options, use `|` to chain multiple), skip (idk what skip is)
  - No cursor management, if needed, modify attributes to manually emulate cursors
  - Need for proper gui feels more and more usefull
- Nodes in the tree are either empty (organizational groupers) or an individual window
  - I'm going to have the organization similar to the tree tab extensions
- Subapps to do:
  - [x] Editor
  - [x] File Manager
  - [ ] Terminal
    - skip for now
  - [ ] Task organizer
- Task system:
  - Just consider 1 file for now, no global, no references between projects, don't even care about the project directory as a whole.
  - store in a json file, edit and view with task organizer
  - [ ] Task
    - [ ] Head
    - [ ] Body
    - [ ] Check
  - [ ] Subtasks
- ui elements
  - textbox
  - tree view
  - Don't need a trait for elements (at least not right now)

---

2024/8/23

Okay, I am at the very very early stage where I can barely say that the individual subapps (text editor, file manager, and task organizer) have enough features such that they can symbolize what they are supposed to be.
I have so many ways of improving them right now, but I know that I will always have ways to improve the details.
The thing I must do now is to work towards my vision of the bigger picture, and first determining what that even is.

Over the past few weeks, as I implemented these subapps, I had time to specify my abstract idea of an "all in one app."
The goal for singularity is to increase the user's productivity.
I've thought a lot about the principle of making things customizable if they can not be perfect.
The main feature that this aligns with this idea is allowing anyone to write subapps.
I want singularity to provide tools for subapps so that all subapps that use those tools to be standardized to some extent.
These tools will usually come in the form of abstraction, like abstracting the UI and organization.
The problem is that I can't consider every single use case, so I am going to start with mine.
My ideal version of singularity would allow me to use it for every single productive thing I can do on my computer.
These are (with overlap):
- coding projects
- note taking
- writing essays
- brainstorming ideas
- making music
- email management
- memory management
- event reminders
- task list
- homework
- small scale time management
- logging
- journaling
- organizing
- web browsing
- reading
- watching videos
- writing proofs
- searching for information (both online and on my system)

The organization system goes like this:
- every project has a specific corresponding project folder containing:
  - core file
    - its project id
    - subprojects
      - the subproject name/id
      - where to find it
    - subapps used
      - subapp name/id
      - where to find it
      - subapp settings
        - standard subapp settings
          - settings that are used by the manager rather than the subapp
          - these settings exist for all subapps
          - ex: their file permissions
        - subapp specific settings
      - NOTE: subapps and their settings are extended to subprojects unless specified otherwise
  - owned files called property for each subapp that requests it
- every subapp has:
  - subapp id
  - the runnable
    - haven't decided on what this is yet
  - dependencies
for a given user, their projects might look like this:
- project: root project (more like user configs)
  - meta:
    - standard: basic color pallete
    - code editor: format on save
  - diary subapp data:
    - I worked on cool coding project (link to cool coding project -> devlog subapp -> bugfix) today
  - children:
    - project: cool coding project
      - devlog subapp data:
        - bugfix
    - project: another coding project
      - meta: code editor: don't format on save
    - project: physics
      - children:
        - project: momentum notes
          - meta:
            - standard: tags: archived
        - project: collision notes
          - notes subapp data:
            - collision preserves momentum (link to momentum notes).

I guess a project file organization standard could be a whole different thing.
But I want to be as unintrusive as possible so people who don't use singularity won't be negatively affected because a project file organization standard does comply with this standard, and vice versa.
Having a single folder with all the singularity stuff would be the best way to do this I think, like a shell.nix file or a .vscode folder.
If people don't want the singularity stuff to bloat their project repository, they can .gitignore it.

Speaking of seperating the roles of singularity, these are the components that are needed to make it work:
- subapps
  - each subapp only talks directly to the manager
  - should not even directly access the filesystem (though I might not be able to force this)
  - but, the manager can then talk to the UI or another subapp or the file system on behalf of the subapp
- UI
  - displays from `SAVDR` (standard abstract visual display representation which is like HTML, should take care of **most** usecases)
  - turn user input into `user input events` and passes it to manager
- manager
  - this is what the core of singularity is; what connects all the components
  - takes care of subapps' permission for files
  - provides the proxy between subapp and ui
- file system
  - where long term data is stored

I am not sure how I will implement subapps to be modified at runtime.
It seems that I am looking for a method of [IPC](https://en.wikipedia.org/wiki/Inter-process_communication) (inter-process comunication)
Here are some possibilities roughly ordered from ideal to horrible:
- dynamic library
- cli
- Manager-subapp communication via sockets
  - unix domain socket
  - still have to figure out initialization
- [Rhai](https://rhai.rs/book/start/index.html)
- shared memory
- message queue
  - i think this is similar to what `ManagerProxy` is doing
- wasm
- Custom language
  - please don't do this
Research:
- https://3tilley.github.io/posts/simple-ipc-ping-pong/
  - goes over many ways of ipc between rust
  - uses shared_memory crate for shared memory
  - uses Commands to spawn rust
- Search `crates.io` for ipc
  - d-bus is a linux tool for ipc
    - is pretty widely used
    - i think it uses servers
    - does not use sockets or shared memory
    - there seem to be people who hate it, but posts about why it is bad are also often met with ppl defending it
    - many crates for it, like `dbus` and `zbus`, both very famous
    - is probably not fast
  - `parity-tokio-ipc`
    - uses unix stream for unix and named pipe for windows so it is flexible
  - `interprocess`
    - idk, not that popular but has a lot of features
    - uses sockets and unix domain socket
- Search crates.io for shared memory
  - `rustix`
    - very popular, has many features, including shm. But, it doesn't focus much on shm and in fact seems to lack documentation.
    - Unless I plan on using its other features (which I might), a shm focused crate would be better
    - Has bad documentation
  - `shared_memory`
    - i mean it is called shared memory
    - needs `raw_sync` crate
    - hasn't been updated in a year
- https://users.rust-lang.org/t/shared-memory-for-interprocess-communication/92408/8
  - Use `pthread_mutex` from `libc` crate
- https://www.youtube.com/watch?v=RtVzlk4om6U
  - uses just the std library, std::process
  - Command to spawn
  - stdin, stdout, stderr pipes for communication
- manual shared memory implementation with no crates
  - it might not be too hard:
  - https://stackoverflow.com/questions/66621363/can-you-cast-a-memory-address-as-a-usize-into-a-reference-with-a-lifetime
  - it will definitely be unsafe but i think i could make it work
- How x window system does client-server communication:
  - I realized that singularity is very similar to a window manager
  - https://en.wikipedia.org/wiki/X_Window_System_core_protocol
  - Wayland (which i use personally) has a [similar article](https://en.wikipedia.org/wiki/Wayland_(protocol)#Wayland_core_interfaces) but it focuses on different things
  - overview section:
    - packets sent via network channel
    - Four types of packets: request (client requests server to do st or requests attributes like window size), reply (server respond to request), event (server informs client about relevant event), error (server tell client that request is invalid)
  - Graphic contexts and fonts:
    - `The client can request a number of graphic operations, such as clearing an area, copying an area into another, drawing points, lines, rectangles, and text. Beside clearing, all operations are possible on all drawables, both windows and pixmaps.`
    - this is pretty wild, i think this means that x window system clients do not get to directly manipulate a buffer, instead needing to request modifications
    - I feel like this would be super super slow, if videos and games are forced to go through this as well. I assume there are other ways to do it as well.
    - wait, i think that is what pixmaps are. I literally read the pixmaps section too, but i misinterpreted it
- Takeaways from Command and pipes attempt:
  - Feels unpredictable
For now, I am going to use Command to spawn and pipes to communicate.
If I need speed, I will look further into shared memory, but I just want it to work right now.
I assume pipes only lets strings or bytes through so I will use serde to send custom types.
In terms of organization, I am going to try turning the main logic stuff into a library and each subapp into their own package with binaries.
I think it is possible to let the manager and main logic have a library and a binary, but if not I will make manager a binary and the main logic a library.

This didn't work, new idea is to try using unix sockets with vanilla rust, but I am open to using rustix. Still spawning with Command.
I will start every message with a length of the actual message.

My ideas are still pretty broad, but I think I can make new progress based on what I wrote so far.
The next step is:
- [ ] implement project organization system
  - [x] (manually) make a test directory
  - [x] make project class and parser
  - [x] make project manager class
    - [x] each instance of project manager corresponds to exactly one project
  - [ ] get task organizer to work with project manager
    - [ ] add a way for subapps to talk to project manager (either replace `ManagerProxy` or make it better)
      - [x] split the subapps from singularity
  - [ ] add project heirarchy
  - [ ] add linking/referencing to task organizer

## Dynamic Subapps Research

2024-09-06

I realized that my research on rust IPC was too specific, what I really care about is just runtime plugins in rust.
I think I saw [this reddit post](https://www.reddit.com/r/rust/comments/144zmwk/how_can_i_add_dynamic_loading_to_do_plugins_for/) already, but I looked through it again and found a tutorial on the very thing I am looking for.

Notes on [the tutorial on rust plugins](https://nullderef.com/series/rust-plugins/):
- Goes over a bunch of ways to do rust plugins
- Dynamic libraries
  - libloading is the main crate, even bevy uses it
- wasm is the other interesting option

I will try libloading, and this time, I will use branches so I don't mess up the main branch.

Speaking of branches, my current plan for branch organization is to follow [this convention](https://medium.com/@abhay.pixolo/naming-conventions-for-git-branches-a-cheatsheet-8549feca2534):
- `main` branch
  - should be thoroughly tested and stable (isn't right now), if someone wanted to use the most up-to-date version of singularity, they would use this
  - ie, this is the most recent stable release
- `dev` branch
  - tested, but not ready for users
- `feature/abc-xyz-0`
  - for new features

There are also bugfix, hotfix, and documentation branches, but I won't use them because it is just me.
In other words, all new branches will be in the form `feature/some-description`

I trust bevy, and its [code for dynamic loading](https://github.com/bevyengine/bevy/blob/v0.5.0/crates/bevy_dynamic_plugin/src/loader.rs) is extremely simple (just 25 lines), so I will try following it.

---

2024-09-07

Wait, NOOOO, bevy's dynamic plugin loading is deprecated and will be removed in the next version (0.15)!

You know what, I have had just about enough of the errors and dead-ends with dynamic plugins,
and in trying to do dynamic plugins, I learned that as long as I have a trait for plugins (in my case subapps),
then adding dynamic loaders for those won't require modifying the existing code much.

## Tabs

2024-09-07

I am going to use the term `tab` for a plugin/subapp/extension/feature that is in charge of exactly one thing being displayed.
For now, I will rename all subapp to tab, because subapp is hard to define.
Tabs will have a corresponding buffer, and tabs can run on their own threads.

I want to ensure that the manager itself never has to wait when the tab is doing something,
and I will do that by making a tab handler that will talk to the tab, which will run on a different thread.
So, the manager will have to just wait for the tab handler, which will be written in either the core or the manager crate.
But, I don't particularly care if the tabs are forced to wait for the manager.

I don't really want to annoy myself with the specifics of IPC again, so I will use a simple and plain message queue.

The mutex might require waiting for it, I am actually not sure.

2024-09-08

I actually love rust, it turns out there is a [chapter in the rust book](https://doc.rust-lang.org/book/ch16-00-concurrency.html) about what I want to do.
Here are my notes:
- Send messages between threads
  - mpsc
    - multiple producer, single consumer
  - multiple tx (transmitter)
  - single rx (reciever)
- Shared state concurrency
  - Share data, with `Arc<Mutex<T>>`

There is also the `async` keyword, which I forgot about.
This also seems usefull.
There is a whole [book](https://rust-lang.github.io/async-book/) on it.

2024-09-10

To recap my problem, what I want is to send:

- Events (Enum) from server to client
- Requests (Enum) from client to server
- Query (Enum) from client to server
  - Response from server to client, want this to correspond to each query
  - Ideally, there would be a way to enforce the type of corresponding query-responses. Like, if there were two queries: GetFloat and GetInteger

I learned that:
```rust
enum Enum {
    A = 0,
    B = 1,
}
```
is a thing.
It only works with isize values, and I think it just has to do with how rust stores enums.
Associated consts are also kind of similar.
But, these aren't exactly what I want so for now, I am going to keep it simple and not enforce type correspondance.
If I am to do this later, I might have an enum for Query-Response type, Query, and Response.
Or, I could have a Query trait with the `T=` thing.

The most ooga booga way of doing this would be to have a mpsc pair for each of the packet types (Event, Request, Query, Response).
I am going to do the ooga booga way, because the logic is going to be abstracted anyways, so I can easily change it later.

---

2024/9/13

I just realized that I might need to have a seperate function and channel for each possible query if I want it to be all type safe.
You know what, I am just going to try doing whatever works, and try not to think about it too hard.

Later on, I might make a macro to automatically make a function for each query-request.

---

2024/9/14

Today, I will work on acually letting tabs display stuff.
Here are the ways I thought of:
- Plain mutex of display buffer without extra logic
  - Works, but if a tab writes for a really long time, then the manager will be stuck. (I am assuming)
- Mutex of display buffer and the manager only updates if the mutex is currently not locked
  - I assume this is possible
  - Is better, but is kind of bad if the tab locks for a long time and unlocks for a short time repeatedly
- Enforce some form of double buffer
  - Only problem I can think of right now is changing the size of the double buffer on manager side
  - I did this, and it works for now.

I think I am done with the tab backend.
The next big category of things to work on is figuring out how projects will work.

<!-- ## Project Organization

2024/9/14

I've already decided on organizing by projects, but within that, I am not sure how to further organize.

So, I am going to start by exploring how other apps do this.

- Nixos
- Obsidian Vaults
  - I've used obsidian for school notes, I liked a lot of things about it and it is a pretty good productivity app
  - Markdown files can link to each other, though I've never done it
  - Tbh, I don't know how Obsidian actually does this stuff
- `.vscode`
  - When you change a setting for a specific project in VSCode, it stores the new settings in `.vscode/settings.json` or something like that
  - This is simple and gets the job done
- Org mode
  - I tried to use Emacs before but I didn't like it (which is partially why I am making this). From what I've heard, one of the key selling points of Emacs is something called Org mode. -->

## Improving UI

2024/9/14

Never mind, I think I should improve the UI first.
The reason why I want to do this is because when I thought about actually using singularity, I realized that I would need to improve the UI (mostly display rather than input) but I could still use singularity if it didn't have project organization.

So, I am going to either look into actual GUIs in rust, or just make a really good TUI.
(Later, I want singularity to have an agnostic UI though.)

### Researching Rust GUIs

2024/9/14

I am not sure if I should do a full GUI or improve the current TUI, but I will research my options.

I want something that easily allows me to designate certain rectangles to different tabs.
Currently, for the TUI, I give a buffer to each tab.

- `wgpu`
  - very fast, might be overkill
    - Looked at code for displaying triangle, definitely overkill
  - bevy uses this
- `gtk` and `glib`
  - made by gnome
  - rust is on their [home page](https://www.gtk.org/), so it probably has relatively good support
  - I might need to install gtk if i want to develop or run this
- `egui`
  - Supposedly the easiest rust gui
- `iced`
  - Compared a lot to egui
- `glutin`
  - BIG ADVANTAGE: can embed Servo, a rust browser engine
- `wayland-client` or `smithay`
  - both surprisingly popular (~10mil downloads)
  - I use wayland and no one else will use singularity anytime soon, but I still want it to be as cross-platform as possible
  - pretty low level

I'm just going to try doing egui and see what happens.

Egui allows custom widgets, so I might be able to use that to split areas for tabs.

---

2024/9/15

Egui isn't really hard, but I feel like it is too much boilerplate and it doesn't have a lot of examples.

I came across `winit` which seems to be a bare-bones thing, and that might actually be better for this purpose so I am going to try that.

Running the most basic winit code: `EventLoop::new().unwrap()` returns an error `error: WaylandError(Connection(NoWaylandLib))`, probably because of some Nixos thing.
An `egui` and Nixos user had [a similar issue](https://github.com/emilk/egui/discussions/1587) and they fixed it with a flake.

Tbh, I don't really feel like actually learning how to use nix flakes right now, so I am going to try changing my entire configuration to fix this.

---

2024/9/16

I couldn't change my entire configuration to fix this, so I just added the flake from github and then did `nix develop`.
I promise I will learn nix flakes, and when I do, I will make my own flake.nix for singularity.

But, the good news is that winit works now on my machine, my code right now doesn't, but I know winit works because when I do `nix develop` on this directory and then go to the directory storing winit then run `cargo run --example window`, then it shows a blank window.

I guess nothing is showing on my code because of [this](https://github.com/rust-windowing/winit/issues/776) bug involving wayland.
Apparently the change to fix this was closed.
The bug is that wayland doesn't show windows until something is drawn on it.

I was trying to draw something really simple just so I could see the window, but as it turns out, that is really hard to do with juset winit.
I guess I will use wgpu then to draw stuff.

Honestly, I might need to make another crate just to abstract all the UI nonsense.
I am hoping this will make development easier, and that it will also help when I make singularity agnostic down the line.
So, I will do that after committing to save progress.

Apparently Cosmic DE uses iced, so I am going to see what iced is like.

### UI Abstraction

2024/9/18

I want there to be a display, which is like the os window.
With a display, I want to be able to be able to split rectangular sections out of it and give those sections to each tab.

Because of rust's mutability safety restrictions, I am considering a system where modifying the a rectangular region doesn't update the display until the display's `fn update(&mut self, region: Region)` is called.
I am not entirely sure what methods are fast enough, so if this is noticably slow, I might have to rewrite everything.

Another way might be to utilize the graphics package (iced)'s existing systems like widgets.

2024/9/20

I am considering an element system, something similar to HTML.
Instead of giving subapps buffers and giving them elements as a tool to modify the buffer, I can force them to use elements by making them return elements or modify elements.

2024/9/23

...it is actually not as simple as I thought.
The three ways I can think of doing elements is:
- Element is data, and shared with mutex. To update, just change data. Could possibly also notify element updates
  - Feels like it should be faster than having immutable data, but ultimately, I am not sure if this much faster. Suppose there is a large nested element and a small part of it is changed. Updating this would be no different than 
- Element is immutable and the tabs need to send a new element every time they want to update
  - Don't like this one
- Element is trait, and shared with box (maybe mutex is also needed)
  - If owned by main app
    - Main app can call an update function of element, and when this is called, it somehow gets data from the tab (reciever, or the main app passes data from tab to the element when calling the function)
    - I assume this is how iced does it
  - If mutex
    - Send data by modifying element

After considering all my options, I am considering either the element is data or element is trait and owned by main app.
I am going to try the data mutex one, and if it doesn't work I will try the trait one.

2024/9/24

Running the UI pauses the thread until the UI is exited, and I can't run winit stuff in a seperate thread either because Mac forces all UI calls to be in the main thread.

The loser way would be to run all the other stuff in a seperate thread, but that is so lame that I would rather just switch back to egui.
Apparently winit has a way of allowing non-main thread execution, but the problem with that is that I don't care.

I might actually need to switch to egui, so I am going to commit before I go any further.

Okay, egui also uses winit, BUT, apparently egui [allows me to easily allow non-main thread running](https://github.com/emilk/egui/discussions/1489).
But, I need to use winit, and eframe 0.28 and winit 0.30 are incompatible or something.

2024/9/25

I added active updates, but currently you need to give it an event to make it update.
I can jankily do continuous updates by just requesting updates on every update.

The basic foundations for all the features (ui elements, ui events) have been implemented, so I am going to implement the following, and as I do that, I can add necessary features:

- [x] text editor
  - [x] more variety of keyboard input
    - [x] ~~modifiers~~
    - [x] ~~more characters~~
    - I just exposed egui's stuff to anything using singularity ui
  - [x] char grid element
    - more or less a tui display, with monospace font and possibly basic visual features like colors
- [x] tab traversal
- [ ] tab selection with mouse
  - for the prototype, just make it so that mouse is used only to select a tab
  - SKIP for now
- [ ] window management with tabs
  - [x] tab's tree hierarchy should not define position
  - [x] tab tree hierarchy should not define display order
  - [x] make tabs able to overlap
  - [ ] able to move tabs around with keyboard
    - [ ] maximize focused tab with `Ctrl+Shift+Up`
    - [x] minimize focused tab with `Ctrl+Shift+Down` (and change focus?)
    - [ ] move focused tab to the side with `Ctrl+Shift+<Right/Left>`
  - [ ] close tabs with `Ctrl+Shift+W`

---

2024/9/27

I was going to implement tab traversal, but egui's philosophy is getting in my way.
I want to know how Zed does gui, so I am reading a [blog from Zed](https://zed.dev/blog/videogame).
- the blog actually mainly goes over gpu programming, and singularity is not ready for that yet.

I guess I will continue using egui, but I will just try to have the backend not matter as much as possible.

---

2024/9/28

Since my philosophies on gui seems to not be shared by other gui frameworks, I will try to go lower level until I can just implement it myself.

Right now, I created a new directory called testing which is completely unrelated to everything else, I am just using it to test how it would be to use wayland-client. If this doesn't work, I might need to do winit+some gpu programming.

Okay, so with just wayland-client (no smithay client toolkit), it definitely feels possible but I think if I tried to rawdog it, I would be wasting my time.
I was looking at [this example](https://github.com/Smithay/wayland-rs/blob/master/wayland-client/examples/simple_window.rs) for wayland client.
Anyways, I am now going to try out smithay client toolkit, and luckily for me, there is an actual [tutorial](https://smithay.github.io/book/client/sctk/environment.html) on this, which I have learned not to expect from most rust crates.

I had to add a few dependencies to even get it to compile, but that is not a big deal.
However, the code from the tutorial is not actually working, and links to the actual documentation in the tutorial are also not working, so I assume that the tutorial is outdated.
Smithay's `Environment`s seems to have been removed since version 0.17, and 0.19 is the current newest.
This is not very reassuring, especially the fact a feature so widely used that it was in the tutorial could be removed in an update.

I feel like this is going to be a whole can of worms, so I don't think I should implement wayland and smithay at this stage.

I will try looking into Zed's gpui, but if I start feeling like it isn't much better than egui, or that it is just too complicated, I am going to commit then undo asap.

I tried it, but there was a very long error message ending with `cannot find -lxcb: No such file or directory collect2: error: ld returned 1 exit status`
and it probably has something to do with Nixos, but I will keep true to my word and abort this tangent before it consumes any more of my time.

---

2024/9/29

Here is a cool snippet: `find . -name "*.rs" -type f -not -path "./target/*" | xargs wc -l`:
- `find .`: list all items in the directory with the following conditions:
  - `-name "*.rs"`: ends with `.rs`
  - `-type f`: is a regular file
  - `-not -path "./target/*"`: is not under the `target` directory
- `xargs`: changes one type of input to another (not rly sure about this one, it just works) in this case, stdin to argument
- `wc`: displays line count, word count, and char count
  - `-l` displays line count only
- In short, this displays how many lines I've written in this directory.

I wrote 3057 lines, 9083 words, and 101606 characters of rust.
The top three biggest files are:

| file            | lc  | wc   | cc    |
| --------------- | --- | ---- | ----- |
| task organizer  | 318 | 1083 | 11996 |
| text box        | 317 | 1006 | 10381 |
| project manager | 313 | 773  | 10997 |

and the top two of these are currently not being used, so my line count is pretty inflated.

### Decoupling Tabs

2024/9/29

In order to seperate the tab tree hierarchy from the view, I will need to redo how I store tabs.

The current way is to only use rooted trees.
This worked when the tree hierarchy determined how things were viewed, but now I also want to set an order to rendering.

My ideas are:
- Store tabs + z-order in tree
  - Sort by this before every render
- Store tabs in tree+store a seperate vector of paths to tabs in order.
- Store tabs in vec, store seperate vector of indices pointing to the tabs for render order, also store the tree hierarchy with tree of indices
  - I like the idea of this, but there is just one change to make this better
  - The problem is that any modification to the order (caused by closing a tab) would mess up everything
- Each instance of a tab has an immutable uid, store tabs in a vec/btreemap, store hierarchy and render order via the id

### Exploring Alternative Frameworks

Egui is not letting me manually set the sizes of widgets.

I am going to try wayland client once more.

In the sctk (smithay client toolkit), there is an example called relative pointer.
I couldn't get the example itself to work, but it uses a crate called raqote and font kit.
I previously could not write actual text, but I guess these crates can render text for me.

For their window example, they use minifb, and the minifb looks very simple and good, but it isn't super widely used, so I'll only look into it if wayland doesn't work.

So right now, for nested items like Containers and Borders, I create an entirely new buffer to store data for it, then I try to copy it onto the parent buffer.
As you can imagine, this is not great for speed.
Actually, nvm the real problem seems to lie with the text, so I was going to optimize the buffer thing but I will fix the text first.

Also, every once in a while, the lengths of the canvas and draw target (which I am using as the buffer) don't match, even though they should be the same thing.
- I think this happens when I resize, but it is non-deterministic.
- Okay, I think it happens when a double buffer is created.
  - All crashes happen on double buffer creation, but double buffers can be created without crashing
- Hmm... a consistent way to crash is by resizing it to the left or right extremes.
  - When breaking this way, it breaks on the time it creates a double buffer
  - Can happen on other resizes though
- The canvas is slightly longer than the draw target. The draw target matches `4*width*height` which is same even when canvas is created. The canvas is greater by a multiple of 4 (from around +4 to +36)
- I fixed it really jankily by not drawing when the sizes are wrong

TODO: keyboard handling abstractions

---

2024/10/01

I am going to try to optimize the displays.
- Currently, for nested elements like containers and borders, I create a new draw target. I did this for "safety," aka ensuring that elements wouldn't draw outside of its given areas.
  - However, I don't need to do that with the current system because I can just make sure the elements follow the rules when they are drawing.
  - I did this, and it is still noticably slow...

To quantify performance, I will log times.
I am running this on my laptop at full screen with quite a few background processes, notably VS Code and Firefox (with most tabs unloaded).

Right now, the general output is roughly:

```log
Starting drawing. 3.991012ms elapsed since last finished drawing.
Starting rendering. 4.025498ms elapsed since last finished drawing.
Started drawing elements. 4.202639ms elapsed since last finished drawing.
Finished drawing elements, starting copy. 398.530553ms elapsed since last finished drawing.
Finished rendering. 399.172312ms elapsed since last finished drawing.
Finished drawing. 399.266227ms elapsed since last finished drawing.
```

This isn't the average or anything, but it gives a good sense of the magnitudes and we can see the rendering taking almost all the time.
I thought copying from dt would take a lot of time, but I was wrong.

My next optimization will be to render fonts once and reuse it instead of rendering it each frame.
It turns out that Font doesn't implement Sync, so I need to pass a fonts parameter whenever I call draw.
Okay, this didn't help much, the result is:

```log
Starting drawing. 4.202717ms elapsed since last finished drawing.
Starting rendering. 4.243526ms elapsed since last finished drawing.
Started drawing elements. 4.736223ms elapsed since last finished drawing.
Finished drawing elements, starting copy. 392.39039ms elapsed since last finished drawing.
Finished rendering. 393.597623ms elapsed since last finished drawing.
Finished drawing. 393.689893ms elapsed since last finished drawing.
```

I might revert this unless I absolutely need to.

```log
Starting drawing. 5.521291ms elapsed since last finished drawing.
Starting rendering. 5.560596ms elapsed since last finished drawing.
Trying to get root element. 6.125869ms elapsed since last finished drawing.
Got root element, starting drawing elements. 6.128967ms elapsed since last finished drawing.
Finished drawing elements, starting copy. 418.385396ms elapsed since last finished drawing.
Finished rendering. 419.475338ms elapsed since last finished drawing.
Finished drawing. 419.533916ms elapsed since last finished drawing.
```

I wondered if locking the mutex might've been holding us back, but it seems to take the least time.
(I already thought of ways to make this efficient. good job, past me)

I made a tool to log, and when I logged, I got:

```log
Starting 'fill rect'...
Finished 'fill rect' in 252.765µs.
Starting 'draw character'...
Finished 'draw character' in 9.04µs.
Starting 'fill rect'...
Finished 'fill rect' in 252.673µs.
Starting 'draw character'...
Finished 'draw character' in 6.743µs.
Finished 'draw char grid' in 367.730814ms.
```

So, it seems that drawing character grids is taking a significant bit of my time.
Specifically, the 'fill rect' is taking enough time that it is noticable when there are potentially thousands of characters being rendered.
I asked google, and 2µs * 1000 = 0.2s = 200ms, so it seems I caught the culprit.

I don't know what is wrong with raqote, but I might need to get lower level than raqote by doing some gpu stuff myself.
While I am on this topic, I want to log an idea I had:
- Chunking
  - Squares (or maybe 1x2 rectangles like a terminal character) of constant size, probably like 8x8 or 16x16
  - Each chunk has an owner
  - More or less a pixel buffer, but with extra steps
  - Benefits:
    - Possibly faster than pixel buffer
    - Feels like an upgraded version of the terminal
  - Problems:
    - Very rigid, can't resize smoothly
    - Possibly slower than pixel buffer because wayland doesn't store data this way
Anyways, here is a roadmap for me:
- [ ] Modify an array of u8 / u32s with gpu
- [ ] Pass data to gpu
- [ ] Render text
- [ ] Implement each element individually

Uhm, this is really awkward, but it suddenly works now.
I wrote some optimizations before writing the previous paragraph, and didn't bother to test it out because I thought it wouldn't work.
I was secretly kind of looking forward to learning gpu, but I guess I can't be complaining.

2024/10/02 future me here, I should have looked further into `tiny skia` instead of trying gpu.
The main page of tiny skia says raqote is very slow, and the benchmarks support that to a high degree.
One problem is that it doesn't have text though, which is a pretty big problem.
Still, my point is that raqote is very slow.

### GPU

2024/10/01

You know what, GPU time.
- [x] Modify a mut slice of u8 / u32s with gpu
- [x] Pass data to gpu
- [ ] Render text
- [ ] Implement each element individually

I think I'll go with vulkan instead of opengl, since it seems Zed and Cosmic DE both use vulkan (can you tell yet that I am unable to form my own opinions?).
The commonly used vulkan crates seem to be:
- ash
- wgpu (not vulkan specific, but supports vulkan)
- vulkano
  - people talk about it a lot, but much more people use ash than vulkano

Actually, smithay has a wayland-egl crate, which I guess will work nicely with everything else from smithay, and it supports both open gl and vulkan apparently.
However, it seems very sparse in documentation.
Also, smithay's own [gpu example](https://github.com/Smithay/client-toolkit/blob/master/examples/wgpu.rs) uses wgpu instead of egl, so egl might not fit my use case.
I am not too worried about integration, since I just need to modify a slice.

All the ash examples I look at use winit, which begs the question of why I chose wayland client over winit.
I might need to migrate to winit later on.

2024/10/02

I looked further into zed, and they use a crate called blade graphics.
It isn't very widely used, but I guess the creators of zed like it.

I think I need to start considering crates with not a lot of downloads.
There is a crate called `ocl`, and I instantly love how simple their example is.
The repo was last updated 6 months ago, so I am not entirely sure if it is 
`rust-gpu` is even simpler, and it actually somehow works in pure rust, which is very cool.
It uses something called spirv, which adds an extra layer of complexity to the code.
Rust gpu also currently doesn't have a crate.io page, which is kind of weird.

I am going to try ocl. 

After a lot of fiddling with nix, `hardware.opengl.extraPackages = with pkgs; [ intel-ocl ];` in my configs.nix is what finally fixed my problem.

Make sure `clinfo` says number of platforms is at least 1 if you are also having issues.

---

The simplest way of sending the element data to the gpu is probably by having a different rendering function per each element primitive (text, rect, ...) and passing the remaining arguments (coords, color, ...) via parameters.
Later on when I do multiple elements, I might just render each element and let elements render on top of each other.

My shader code is most definitely suboptimal, but it is a good experience probably.

Wow, I don't understand half of what I just "wrote" but it was surprisingly not as hard as I thought.
The problem right now is that I don't understand the types and how to pass data to the gpu.
To fix this, I will read [this tutorial on OpenCL](https://www.nersc.gov/assets/pubs_presos/MattsonTutorialSC14.pdf).
This is what I gathered from the tutorial as well as the example code from ocl.
- Levels of stuff (pg 11):
  - Host calls the compute device, which consists of multiple work groups, which themselves consist of multiple work items. Each work item calls the kernel function once.
  - Levels of memory:
    - Host memory
    - Global and constant memory: shared within the entire compute device
    - Local memory: shared within a work group
    - Private memory: individual for each work item
    - This is pretty helpful, so I guess inputs marked as `__private` are individual for each work item, and the same thing for the other levels.
- The cl kernel code starts at pg 41.
- Vectors (eg `int4` is 4 integers, more like const sized arrays than rust vecs)
  - I needed a seperate auxillary crate `ocl-core-vector` to send vectors as args
- NOTE: `float` is rust f32
- NOTE: when a type is marked with `*`, it is actually just a pointer so it would be equivalent to a rust `&` I think
- `get_global_id(0)` returns the first work id, which is the x pixel in ocl. 1 is y.
- NOTE: Ocl's `ImageChannelDataType::UnormInt8` is a float from 0 to 1, not an integer, to use u8 is: `ImageChannelDataType::UnsignedInt8`

[Here](https://registry.khronos.org/OpenCL/specs/3.0-unified/html/OpenCL_C.html) is the documentation for open cl.

---

Idk how I would start displaying fonts, I guess I should start with more research.

https://en.wikipedia.org/wiki/Computer_font
- Three basic ways to store each glyph:
  - Bitmap
    - Matrix of pixels
    - Feels jank even for my standards
  - Vector/outline
    - Store instructions on how to draw, like bezier curves
    - I don't want to manually do all the instructions for this
  - Stroke
    - Store a series of strokes (and possibly other info)

I actually don't want to do fonts myself.

`ab_glyph` and `rusttype` both seem  to meet my needs.
It looks like ab glyph made rusttype obsolete, so I guess I am going with ab glyph.\

Actually, I could use ab glyph's rasterizer to support the gpu.

Okay, it seems pretty simple, but it is a little annoying that I need a .otf file in the project to do this.

font_kit, the crate I was previously using, might actually work, and it doesn't require me to include a font with the project.
It is tightly coupled with the pathfinder crate, which makes sense because both crates are developed by the servo project.

2024/10/03

If I need to import a font, I'll go with DejaVu fonts because it is public domain.

---

I got ab_glyphs to work, but my implementation isn't super fast.
Of course, loading takes time, but even for just the drawing portion:
```rust
q.draw(|x, y, c| {
    *img.get_pixel_mut(x + 10, y + 10) =
        Rgba([(c * 255.) as u8, 0, 0, u8::MAX]);
});
```
drawing a 12pt character onto the picture takes around 20µs.
For a glyph, raqote (criticized for its speed) took around 5-10µs.

As for the gpu renders, building a kernel took around 150µs and executing it took around 20µs.
Raqote took around 250µs for rectangles before optimizations
So, I hypothesize that the more things I can do with a single call to the gpu, the more effecient it will get compared to the cpu.

Okay, knowing all this, I am going to put a hold on the GPU stuff, because it wasn't as fun as I thought it would be.

---

2024/10/03

I think that the best thing now is to just implement the other subapps.

Once I do that, I can start the project/workspace wise features.

### Hierarchy Operations

2024/10/08

I have some ideas for moving tabs in the hierarchy.
I think the easiest way is to "<u>P</u>luck" trees into a temp buffer, then "<u>P</u>lace" them somewhere else.
BTW, I am going to be using Alt for most (or all) the hierarchy operations, because that is how window switching works in other OS's (Alt+Tab).
There can also be a mark operation to allow for things like swapping two tabs.

Before that, I should actually do tab closing first.
The one tricky decision with closing is the tree hierarchy.
Ideally, the tree (specifically the deleted node's children) would maintain its rough structure when a node is deleted, and a parent node can be deleted without deleting the children.
The two basic ways of doing this would be:

1. Put all the (direct) children of the deleted node in the 
   1. If thinking connections-wise, equivalent to just connecting deleted node's parent directly to deleted node's children.
2. Make the first child take the place of the deleted node, and the other children don't move so they are now the children of the first child
   1. Kind of similar to the heap deletion algorithm.

I will first implement a full deletion first, but I think it will be best to implement trees specific to my goals.

2024/10/10

I am implementing a tree based of id's, but something I might want to change later is the fact that the id tree is heavily coupled with `Tabs`.
It is like the rooted tree, but it just uses IDs as indices.

Deletion is actually quite hard, on second thought, I will not do hierarchy operations right now.
(Ex: removing a non-last sibling requires all later siblings and their children to redo their paths, I might end up needing a different system of storage entirely. The fact that children are ordered is a big obstacle)

## Organization

2024/10/10

I was thinking about my vision for singularity, and I think this is the order of "categories" that makes the most sense to me (think of this like the Kingdom-P-C-O-Family-Genus-Species):

1. User
   1. The file storing the user data could be a special case of the project storing file, maybe with a special `userfile=true`
   2. But, there could be shared projects later on
2. Project
3. Custom User-set hierarchy between tabs

This is similar to how in nix, there is a configuration.nix which is for the user (or the machine), but when a user enters a nix-shell or dev flake, the packages/features available are a union of the packages from configuration.nix and shell.nix.
Actually, now that I think about it, most apps are like this.
As another example, VSCode has User Settings and Project Settings.

2024/10/11

In order to code faster, I am going to ignore the permission stuff for tabs, which seems like bad foreshadowing.
Additionally, I will ignore things like abstraction, since the main priority is just the task organizer.

2024/10/16

I want to be able to standardize children elements (eg: components or widgets) for things like "forwarding" events (especially mouse-clicks) and possibly focus.
Planned steps:

1. Make a trait for elements
   1. `Tab` kind of works for this, but something else might be better
2. Create forwarding method
   1. Only difference right now between `forward_event` and `handle_event` should be that `forward_event` somehow should take care of mapping mouse

2024/10/17

Ugh, I don't know how I want to do this...
I might somehow make use of Mutexes of UIElements.

Actually, it was pretty easy, barely an inconvenience.
I did it with a new trait called `Component`

Things I could implement/consider for the components stuff:
- [ ] focus (should I even do this?)
  - [ ] either make this an event (this is probably the way most other things do this), or give `focused` as a parameter for each render, or both
  - [ ] implement setting focus
  - [ ] implement different display for focused vs unfocused
- [ ] vec of components as a component (like UIElement::Container)
  - [ ] Somehow make this like map tuples and stuff, (like being able to write `Components<(A, B, C)>` and it works like a vec for Components, but it works like a tuple from the outside (you can call `component_bundle.0` and the compiler knows it is type `A`))
    - [ ] Might be able to do this inductively, inspired by: [tuple tricks crate](https://crates.io/crates/tuple_tricks)
    - [ ] Recursive Impls are probably better, it is also in the [std library](https://doc.rust-lang.org/src/core/fmt/mod.rs.html#2628)
      - [ ] Same thing, but [simple example](https://stackoverflow.com/questions/55553281/is-it-possible-to-automatically-implement-a-trait-for-any-tuple-that-is-made-up)
  - [ ] get focus working with this
- [ ] make the components with inner components less bulky to write
  - [ ] maybe mutexes might help?

2024/10/24

I don't like the way this is currently implemented, but since I am using macros anyways, I might look into having macros instead of using the complex types.

I want to reattempt tab structure operations.
My first change is going to have the tree path be not order based, but id based.
This means I will need to reimplement all the tree traversal logic and very tightly couple the tree logic with the tab logic, but I am willing to make that sacrifice.

2024/10/25

Btw, I ended up having tree logic decoupled from all the tab stuff.

Anyways, I am making procedural macros, and it was kind of annoying to start because there was not great documentation, but it feels pretty simple once you get started.
Here are my tips:

- My main source for getting started was a [video](https://youtu.be/crWfcA064is?si=AbTf290vzLE4bhR0) by Let's Get Rusty.
- Use `cargo expand`
- [this article](https://www.freecodecamp.org/news/procedural-macros-in-rust/) also seems good and very in-depth
- use the `quote` macro


2024/10/27

I've been going on tangents from the `Organization` subsection, but I will get back on topic now.
The thing I was supposed to improve was the `.project` stuff, with an initial focus on tasks.
I think I will just start "drawing inspiration" from similar existing tools like VSCode and Nixos.

First, I want to improve open/close behavior:
- [x] be able to run and specify project path in cli, something like `singularity_manager --project examples/root-project`
- [x] save the workspace's open tabs on close

2024/10/28

I am working on saving tab sessions on close, but how should the data transfer happen?
I could just do something rather---contrived, if I only cared about this, but I would be avoiding the overarching matter of how information should travel between the tabs and the project.
And in a way, this brings into question what singularity itself is.

I initially thought singularity should simply be an app to host other apps.
I still want an app that does that.
But after inspiration from Wayland, I believe that an interface protocol between subapps/tabs and a centralized app can be much more powerful.

Anyways, that is something I should keep in the back of my mind while I continue making progress.

I think I can draw inspiration from webpages for saving data.
These are the aspects of webpages I think are relavent:
- Information for opening a webpage:
  - The webpage location in the url (most of the url)
    - Parallel the "tab_type"
  - Extra parameters in the url (like the `url.link/page?parameter`)
    - Examples of this are in many search pages and also when specifying sections (wikipedia)
    - Data from the opener to the tab on initialization
    - Should also remember/ask for this when saving a session, if you want to restore it later
    - Already implemented this for opening, but I should also add it for closing
  - Cookies/local storage/session storage
    - Local storage
      - Data per tab type (and per .project type)
      - I think I can add this with queries
    - Session storage
      - Data per instance of tab
      - This is just variables, already implemented

2024/10/29

I can add initialization parameters and something like local storage through queries.
In essence, the initialization parameters are storing the session storage long term, from one close to the next init.

Both extra parameters and local storage will not be generic types that are different per tab type.
Instead, I will pull a javascript and just store a serde value or even just a string.

I think there is a reason why webpages don't save data on closing like this (that I know of, +other than caching).
Saving on close is a pain, especially with the current architecture with threads.
Keeping with the webpage inspiration/theme, I think I will just save the initialization data throughout, and save it on close.
Later on, I can modify that code slightly to have a more general tab data.
Wait, I think I just reinvented session storage.
(As much as I like to hate on JS and webdev standards, I am slowly recreating it with this project...)

Though I could use Mutex for session storage (and later might), I will do it with queries for now.
If I do that, the current infastructure for UIDisplay can be reused for session storage.

I got expected behavior first try!
However, I still need to find a way to figure out the tab type.
In fact, I am not sure how I am going to handle tab type at all.
But, I am ready to sleep now.

2024/11/1

Coder's worst nightmare, naming variables, strikes once again.
I've been stuck working on automatically generating code for queries.
I am trying to procedurally name variables, but that has turned out to be quite a hassle.

Concat idents is unstable, and while the paste crate seems cool, but is deprecated for some reason.
I tried to do my own implementation of the concat_idents macro, but it sometimes just doesn't work, so I will try using paste even though it is deprecated.

Using paste, I automated generating the query type stuff, but it is not very readable, and I am also worried about its performance.
Additionally, if something does go wrong, it will be a nightnmare to debug.
I haven't used the auto-generated queries yet.

Uhm, this feels noticably slower, I can't remember if it is because of my new changes though.

2024/11/2

I think that was just because I had so many tabs open from the open/close session test.

2024/11/3

I tried to do wayland embedding in another branch, but that didn't work out.

Anyways, I want to work on organization in the form of tiling.
I will ~~copy~~ gain inspiration from existing tiling window managers like hyprland.

The problem is that a lot of the keybinds I want to use are already being used by KDE.
I am going to disable them for my entire system, because there is no way to selectively change KDE keybinds based on app focus.

2024/11/4

I talked to a friend (@glolichen) who uses Hyprland, and it seems like everything can be stored in a binary tree, with leaves being the actual windows, and non-leaf nodes storing two children, and data about how they are organized (eg, hor vs vert split, split ratio).

2024/11/6

I am once again turning to Uuids for the binary trees, and I realized I wanted type safety with uuids, but the rust compiler is kind of annoying so I might use an external crate for this.
I looked at the `typed_id` crate, and it just manually implements what would have been derived.
But, I feel like there is something wrong with my logic as I rely more and more on Uuid's.

2024/11/7

I learned that hyprland has two ways of doing tiling: dwindle and master.
Dwindle is what I was basing my tiling off of.

2024/11/11 (technically 2024/11/12 1:09 AM)

Plan:
- [x] render based on focus state (boolean) (this sets up tree elements ui to change highlights on select)
  - ideas
    - pass `focused` boolean on render (idk, feels kinda jank)
    - pass render_data on render (a more abstract version of the `focused` idea)
    - use macros (this will be the hardest, but I want to do it)
- [x] some abstraction for tree displaying
- [ ] improve task organizer

2024/11/12

I think I was overcomplicating it.
For tab focus, I need to standardize it, but with widgets, I might just have it be different for each.
If I really wanted some standardization for widget focus, I could just add a trait.

For the tree displaying thing, I will use macros.

2024/11/13

I think I can just add it to compose components.

First, I implemented the logic without macros, then, in demo, I wrote what I wanted the macro use to look like, and then I implemented macros while changing the macro use to fit logistical constraints.

(warning: the demo window has an empty background, and at first, it looks like it isn't running)

2024/11/16

I was using enclosed component along with compose component for the focus task editor, so there might be leftover redundant code from doing that.

2024/11/17

I tried to use lldb in VsCode, but unfortunately it requires extra setup in nix which doesn't seem worth my current time.
Plus, it probably would have not worked with multithreading anyways.

2024/11/18

Bruh, why did I never find [this video](https://www.youtube.com/watch?v=Api6dFMlxAA)?
The title isn't really descriptive but it is a video on tiling wm algorithms.
Things I want to look for:
- Moving tiles around
  - (Representing tile structure and adding and deleting tiles are already implemented and are pretty intuitive)
  - The current swapping system is limited
- Inspiration for tab hierarchy as well if possible

Notes:

- 5:32 starts talking about approches to tiling
  - 6:14 list-based
    - 6:35 stack mode/master mode
      - One master takes up half of the screen
      - All others are just stacked on top of each other
      - Lame (imo)
    - 8:20 Max/ful/lmonocle
      - Bruh
      - Sucks
      - Only full screen
    - Continues talking about other stuff, but as expected, all the list-based methods are cringe
  - 11:45 Tree-based
    - The example's tree structure is similar to what I use and has parents being containers while leaves are actual windows (tiles in my case), but it is not a binary tree
    - I didn't watch the whole thing, but unfortunately, it seems the slides only go over adding tiles so it doesn't really help me

I tried to look elsewhere for similar sources, but could not find anything.

For moving tabs within the tab hierarchy, I think I have a way:
I already have tab hierarchy traversal, where given the current tab path and a keybind (Alt+some char), I get a new tab path.
I can have a corresponding keybind for each traversal (Mod+Alt+same char) (I wanted to do Alt+Shift, but that would make things harder bc it would modify the character) where the move keybind just swaps the current path's tab and the new path's tab in the tab hierarchy.

Actually, while implementing the above method, I realized that there was already an easy way of implementing this.
There was already a selecting screen, so algorithms like pluck and place as well as swapping will be obvious.
But, I will first implement and commit the other way because of sunk cost fallacy.

Gee, I am really struggling with swapping two nodes in a doubly-linked tree (doubly linked referring to the fact that parents and children both link to each other)...
When they are unrelated or are siblings, everything works normally, but with parent-child, everything breaks apart.
I might need to go back to baisics by considering how to swap a doubly-linked list.

2024/11/19

I just drew the connection diagrams on a whiteboard and tried algorithms until I found one that worked:
1. Update all the children connections (for all children connections, replace A and B)
   1. Swap the children A and B for their parents
   2. Swap A and B's children
2. Update all parent connections to match the children connections

My next goal is to get the Task Organizer to a very good point.
I will brainstorm now:

Though have left Java and many of its ideals of OOP long ago, I believe that approaching the Task Organizer in an MVVM (model, view, view-model) approach might be an elegant way of thinking about it.

- Model
  - The first component of MVVM is the model, which refers to how the data is represented.
  - In the case of the Task Organizer, the model will store the tasks and all relevant data.
  - Most of the model can change as I work on the view and the view-model, but an important task regarding the model is file representation.
    - So far, I overlooked the file representation of everything by making rust structures and using Serde to convert data into JSON.
    - For now, there are more urgent matters, but I ultimately want people to be able to edit these files manually with a text editor.
    - Until then, the graphical editor will suffice.
  - Model could also include some logic, but I will be referring to only the representation of the data as the model.
- View
  - The view refers to the UI, including displaying data to the user and recieving user input.
- View-Model
  - The View-Model can be thought of as the connection between the View and the Model.
  - A critical aspect of the View-Model's responsibility is modifying data.

Now that I wrote all that, I am not exactly sure why I did, but I guess it is a good reminder.

Features I want to work on:

- Time management
  - Regulate and log time spent on each task
  - Different from deadlines but I should do that later as well
  - Regulate time with blocking method
  - Log time by recording all timer related actions along with when that action happened
- Templates/repeating task types
- Links
- Online webpage access

Blocking method idea:
I haven't tested this so I don't know if it is actually efficient, but the idea is to work on a task for specific blocks at a time.
A block is a continuous period of time dedicated to a single purpose (like working on a task).
A standard block should have a set-up, work, and clean-up period.
The setup would include setting up prerequisites for the work, finding a good playlist, using the bathroom, clearing the mind of all else (ex: check gmail to not worry about missing emails), and setting up the task and block in the task organizer.
During the setup period, make the work period as smooth and productive as possible.
Cool down during the clean-up period, and take care of the physical workspace so that whatever comes next can be started in a fresh state.
For an hour-long block, a good delegation of times could be a 5 min, 50 min, and 5 min, respectively.
Unfortunately, school and other obligations exist, so I can not divide my life into perfect 1-hour blocks.
I also must consider the fact that the time it takes most tasks (often tasks with objective requirements) can not be perfectly predicted, let alone fit into 50 minutes.

Actually, I am going to just test different ways of being productive and try something new each time I don't like something.

### Trials

Control Trial

For my first trial with tasks, I will start with blocks and a timer that shows overall time spent on each task and logs each start and stop time (can only be viewed from the JSON, meant for future data analysis or debugging) with no time limits or target times.

Future features that I want to test in trials that are not in this trial:
- Reusable block and task templates
  - Ex of block template:
    - Set-up, work, and clean-up
- Pacer
  - Flexible target time
- Try seperating blocks and tasks entirely
  - Task can be analogous to GH issue and block can be analogous to a commit
  - Blocks don't necessarily need to be under a task, but blocks can reference tasks in descriptions (think how descriptions will work)
  - If this test produces fruitful results, could later even have a different app (tab) just for blocks, where references between each are allowed
  - Sub feature to test: descriptions that are like commit descriptions
    - Could have something like a small devlog accompany each block (would kill 2 birds with 1 stone bc I wouldn't need to build a whole new app for devlog and I can save redundant time that would be spent on writing devlogs bc it automatically stores it)
    - Can reference tasks and other stuff (much later, I should standardize linking)
    - Can also have special regions for the description or just templates for descriptions
      - Like a goal section and reflection section
    - Having this will allow users to work on multiple related tasks in one block, and this might be bad if strictness is desired, but I hope it will strike a balance between flow and structure, because it all
      - Having subblocks would be better organizationally, but that is getting out of hand, even for me (if I did this, it would be supplimentary to block descriptions)
- Rigid predefined blocks
  - Set target time limit before each block, and the block must end when the timer goes off
  - If you don't finish by then, start a new block either right afterwards or after doing somthing else
  - I don't really like this

I want to try all these in different branches to test all seperately, but I also don't want to bloat up the number of branches.
Do I make a fork? Can github issues or some other GH feature help me? I guess the feature I am looking for is literally just branches.

Features to add after all the trials are done:
- Blocks report
  - Overview of task worked on at each point in the day per project
  - Have this work with nested projects as well
  - Have a complete overview for all projects for a single person
    - Can also have a special thing to note non-project blocks like sleeping and eating

Many of the features promote data collection at the sake of short term flow, and I am actually fine with that because it means I can continuously learn from this data and improve singularity, and I also predict that being able to visualize productivity will help gamify productiveness.
TBH, I just like seeing data.
I am not worried about privacy, because 1: I might be the only one who uses singularity, 2: this is all locally run, 3: I can add opt-out/in and encryption later if needed.

By the end of the trials, I want to be able to use singularity because it is good.

2024/11/20 12:13AM

NOOOOOO, I thought of the name `chronoschism` meaning time (`chrono`) and divide (`schism`), but it turns out that someone else came up with it first!!!!
Grrr...

I am going to use that name though because I came up with it seperately and it is cool.

Also, I was thinking about the trials, and I realized I am not going to do that in the scientist way; instead of testing all the variables seperately, I am going to use an iterative approach and just use common sense.

2024/11/21

I am going to make a seperate subapp for blocks, called time manager.
The name chronoschism would be better for a name of an idea or game, but it doesn't feel like it should be a name of a tool.

2024/11/23

I had to somehow start a time manager tab, but I don't have a way of starting tabs (TODO), so I temporarily made tabs public and inside the manager's testing code, I made it add a new time manager, then ran it once, and removed it.

2024/11/25

Should making borders around each tab be a responsibility of the tab or the manager?
Right now, I am putting borders around each tab's inner elements in the tab's code.

2024/11/26

Ideas relating to what I am implementing right now (the time manager/the block thingy):
- Log every action saved
  - Like git but it commits when you save (every keystroke is crazy even for me), modify tasks, execute cli command (like a more verbose/intrusive version of `history`), etc
  - Possible rule of thumb: at least every time "data" is modified
  - PRO: Would help automate blocks
    - Instead of starting a timer, doing stuff, and stopping, you could just do a bunch of stuff, and the time manager automatically clusters into groups, like "N saves, cli commands, and etc all with a maximum of a T minute gap between each consecutive action? sounds like a block just happened from x to z O'clock"
  - PRO: Would also be useful if connected to internet
    - Allow for backups
    - Like a middle ground between google docs and git commits (just me, or does anyone else get git commit anxiety, like a feeling of paranoia that a commit is too small: "oh no, people are gonna think I am just farming my github statistics" or simply not wanting to name an insignificant change so I sneak it in with a major change under the name of that change)
      - Google docs reminds me that a good alternative name for "block" is "session"
    - Should figure out how to make this work with git
      - Maybe all the block stuff (and even the .project directory) should be .gitignore'd, especially considering multiple branches or multiple collaborators
    - Connect to self-hosted server?
      - Would be jank, but highly customizable, could have a web interface so it is accessible from my phone
  - PRO: yay data
  - CON: Too much data?
    - Hard to store
    - Intrusive
  - would have to actually implement it (side effects)
    - there would need to be some standardized system for 
      - good bc it could potentially be useful for other stuff
      - bad bc sounds like a pain
    - Customizable user scripts for clustering also sounds fun to use but like a pain to make
- Block notes and title
  - When should I let the user edit this?
    - Before starting and during the block
    - Do I let the user edit afterwards?
  - Notes can be optional
  - Title defaults to "Block N" for the N-th block
- Linking between tabs
  - To quote Kylo Ren: "I know what I have to do but I don't know if I have the strength to do it."
  - Prerequesite to this would be some standardization to the list of 
  - Implementation idea based on the web:
    - Each tab type is like a server, imagine task-organizer.net and time-manager.net
    - That is, there are tab-wise (server-wise)
    - Each instance of a tab is like a webpage session
    - Ex workflow: If an open time manager tab wants to focus a specific task by id, then the task organizer would create a task-organizer packet object with the relevant data (probably an enum like `Packet::FocusTask(task_id)`), serialize it into a json object (or some other standardized but versatile datatype), then tell the manager to forward the json packet to the task organizer tab type/"server" (figure out how to identify tab types, ideally something better than just mapping string to impls of some tab trait). On the "server"-side, somehow try to see if there is already a task-organizer tab open. If so, then somehow identify it and tell it to focus on the relevant task. If there are no task organizer tabs open, then ask manager to create one with the focus on the relevant task.

2024/11/30

I have previously expressed discontent towards my current procedural macro implementation for components.
I want the macro to be more versatile, I am thinking the user writes something like:

```rust
#[derive(Components)]
struct Tab {
    component_field!(some_component_name: String; 
        clickable,
        area = DisplayArea::FULL,
        render = UIElement::CharGrid(CharGrid::from(self.some_component_name)),
        event_handle = { ... },
    ),
}
```
which gets expanded to
```rust
struct Tab {
    some_component_name: String,
    /// Generated by macro `component_field` and not to be used directly
    __some_component_name_clicked: bool,
}
#[automatically_generated]
impl Tab {
    fn some_component_name_clicked(&mut self) -> bool { ... }
    fn render_some_component_name(&mut self, manager_handler: &ManagerHandler) { ... }
    fn handle_event_some_component_name(&mut self, event: Event, manager_handler: &ManagerHandler) { ... }
}
```

The macro to generate more fields was cool, but unnecessary.

Actually, I can just have this, which in terms of the macro formatting, is very similar to the current system (that is to say, the difference is in what it gets turned into):

```rust
#[derive(Components)]
struct Tab {
    #[component(area = DisplayArea::FULL)]
    some_component_name: String,
}
```
which gets expanded to
```rust
...

#[automatically_generated]
impl Tab {
    fn remap_event_some_component_name(event: Event) -> Option<Event> { ... }
    fn contain_some_component_name_render(render: UIElement) -> UIElement { ... }
}
```

hmmm...
I don't want to say this, but how much of this is actually necessary?

Let me take a step back:
Once the excess (uselessly rigid) parts of the macro are removed, it seems all I am doing with the macro is ensuring that clicking and rendering are remapped by the same amount.
And I only do this because I want components, tabs (tabs are slightly different), elements, etc to be allowed to be unaware of the context it exists inside.
So far, context really just means its display area, and the two things impacted by display area is clicking and rendering.
Even its pixel size doesn't need to be known for most purposes because of the relative unit.
There is also focus, but I didn't really implement that, so it doesn't count right now.
Anyways, this means that the burden of contextualizing lies on the container (the user of a component).
This approach will allow components to do the bare minimum, but might create a lot of bloat on the container side.

The meta purpose of the `Components` proc macro (which I might call the `ComponentsContainer`) is to create a flexible tool to help simplify as many container usecases as possible.

Honestly, I might just make containers manually do this like:

```rust
struct Tab {
    some_component_name: String,
}
impl Tab {
    const SOME_COMPONENT_NAME_AREA: DisplayArea = DisplayArea::FULL;

    fn render(&mut self) -> UIElement {
        { todo!() }.contained(Self::SOME_COMPONENT_NAME_AREA)
    }

    fn handle_event(&mut self, event: Event) {
        if let Some(some_component_remapped_event) = event.remap(Self::SOME_COMOPONENT_NAME_AREA) {
            todo!()
        }
    }
}
```

This is very boring but solid.
On the plus side, I should be able to do this already.
I will deprecate the `ComposeComponents` macro for now, RIP ComposeComponents.

Button also seems unnecessary, but I will tackle one problem at a time.

...

I removed the compose components macro from time manager, but I am noticing a few patterns. 
In fact, I might venture to call parts of it "repetitive" or even "automatable".
In case the file has been changed and you don't know what I mean, this is the github permalink:

https://github.com/mathkimchi/singularity/blob/0d16c4488b81f92a99c49e59d14d46202824f3e5/singularity_standard_tabs/src/time_manager/mod.rs#L213-L246

(speaking of cool github features, I should start using GH issues and pull-requests; In the 2024/11/19 DEVLOG entry, I allude to wanting a feature that works just like this)

Well, just look at these three specific snippets in the same file:

```rust
Focus::Title => Self::TITLE_EDITOR_AREA,
Focus::Body => Self::BODY_EDITOR_AREA,
Focus::Timer => Self::TIMER_BUTTON_AREA,
``` 

```rust
else if let Some(remapped_event) = event.remap(Self::TITLE_EDITOR_AREA) {
    self.focus = Focus::Title;
    remapped_event
} else if let Some(remapped_event) = event.remap(Self::BODY_EDITOR_AREA) {
    self.focus = Focus::Body;
    remapped_event
} else if let Some(remapped_event) = event.remap(Self::TIMER_BUTTON_AREA) {
    self.focus = Focus::Timer;
    remapped_event
}
```

```rust
Focus::Title => {
    self.title_editor.handle_event(remapped_event);
}
Focus::Body => {
    self.body_editor.handle_event(remapped_event);
}
Focus::Timer => {
    self.start_button.handle_event(remapped_event);
}
```

There is a little voice telling me this is going to be not as good as I think, that I am overcomplicating this, but a meta macro could work.

The big observation is that there are distinct triplets of a `Focus` variant, a field, and a DisplayArea.

You know what, I am going to leave it at the observation for now, because I want to make fast changes today.
This is definitely a TODO or REVIEW though.

TODO

2024/12/2

Do you see this?
I finally cleaned up the git branches and got around to using issues.

From now on, I will put the technical brainstorming and ideas in github issues.
In this file, until I actually deem singularity (specifically the time manager's block logging) good enough to actually start using, I will mostly be writing session-related things in here.
This will be things like:
- Writing down/logging issues I work on like I will right below
- Journalling about the coding process, like writing down how I got over a specific problem, or if I didn't get over a problem, just ranting about it
- Whatever miscellaneous things I feel like writing here

The purpose of doing this is so there is a central place to see everything I did.

I will work on this issue (hopefully github recognizes this):

#3

...

Nope, but this should work: I will work on [#3](https://github.com/mathkimchi/singularity/issues/3) (apparently vscode recognizes these issues now, because it autocompleted).

...

Beware: Rambling (even more so than usual)

My roommate is sleeping and I wanted to log my thoughts before I wash them away in slumber, so I have resorted to editing markdown on github mobile.
I was thinking of having each process create as many windows as they wanted, if any.
But, instinctually, I worry that jumping to this solution might not be the broghtest idea.
I keep thinking of rust's ownership, not necessarily because the its solution to memory management might cleanly parallel a good solution to this problem, but because it is just so clever that I want to know how that idea was even conceived.
In other words, I want to try to use the example of ownership to learn how to come up with good solutions.
Like manual memory management, my idea pretty much maximizes the flexibility of how tabs are implemented.
Flexibility is great, but in this case, making tabs too flexible will make it unsafe and hard to develop, like C.
Moreover, learning about wayland has shown me that imposing logically arbitrary but practically useful works, which is reassuring especially since I intend to implement a system with the same functionality as wayland.
(However, learning anout wayland has also taught me that it is a nightmare, so maybe it isn't the best role model.)

The thought that initially motivated this was rust's mpsc.
A rust message channel can have multiple senders and one receiver.
There can only be one receiver, because the message can only have one owner.
This is very intuitive if we think of a real mail.
A channel is like a mailing address; there is no limit to how many people can send messages, but each mailing address has one house it refers to.
Rust ownership makes more sense than loose memory management in the real world, even though following it sets a self imposed restriction that isn't necessary by the nature of where code lives.

I wonder if I can draw any inspiration from real life to come up with a good solution.
The takeaway from the above paragraphs is that the inspiration can be arbitrary from a purely logical view (in other words: restrict what the user (a developer using this code) can do even though allowing it would not be hard to implement), if it benefits safety or improves the usecases that will be allowed.
I guess all that just says that guardrails are fine.

2024/12/4

I haven't found a better solution than processes requesting to make windows.
This is the most flexible way.

But, I was thinking more and eventually, I should abstract away everything arbitrary, and for any feature like mouse click listening, have it work similarly to importing libraries.

On account of my desire to open source this project by the end of December combined with my inability to dedicate myself wholly to this project at this moment in time due to urgent priorities (studying), I will continue to brainstorm until Saturday.
On Saturday, I start implementing the least bad solution, even if it is the flexible way.
This will let me:

1. Prioritize my studies for the week, which is the most crucial period
2. Brainstorm and ensure there are no glaring issues with whatever I settle on, so I probably don't prematurely work on a fundamentally flawed implementation (which I have done)
3. Not get stuck on brainstorming, constantly doubting any solution I might do, and never start trying (which I have also done)

2024/12/7

A terminal app called Zellij is written in rust and supports rust plugins, but it is with WASM.
Check out: https://zellij.dev/tutorials/developing-a-rust-plugin/

Anyways, my deadline is up.

I thought about the best ways of doing this, and I didn't get any big revelations.
I will take it simple and do things similarly to window managers and desktop apps: each process can request an arbitrary amount of windows.

I think I will have a singularity client toolkit.
This is similar to the smithay client toolkit, and it is my attempt at making things safe when safety isn't guranteed with ipc.
In theory, if the client toolkit and the server side are both working, then the actual client code should be safe.

There is a [smithay handbook](https://smithay.github.io/book/client/general/intro.html) that I will gain inspiration from.
In wayland, the wayland server has a listener at `$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY`, and if you `echo $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY`, you should get something like `/run/user/1000/wayland-0`.
So, I will have my own env variable called `$SINGULARITY_SERVER`, and the singularity server will make a socket at `$XDG_RUNTIME_DIR/$SINGULARITY_SERVER`.

When the wayland client wants to make a window, they call `Connection::connect_to_env()`.
Each `Connection` should represent one wayland window.
I'll do something similar.

2024/12/13

I haven't worked on this over the week, but it is the weekend, and I want a demo just for the unix sockets.
I will make a chatting app to start simple with.

I also think I should seperate singularity into:

- singularity tab organizer (what I created this project with the intent of making)
- singularity window manager/compositor
- individual standard tabs
  - Chro (name in progress), the time manager
  - rest of the tabs (file manager, text editor, eg.) are whatever, just make them usable

unix-stream seems to be the same for the server and client side, the process just seems to be:
1. server makes a `UnixListener` (this is only for server side)
2. client tries to connect
3. server accepts, and the OS or whatever gives both the server and client each a `UnixSocket` that just communicates between the two

I remember making a multiplayer game with TCP, and I think this is very similar.

A group chat server is unnecessarily complicated, I will just do a simple turn based chat like thing between just client and server.

2024/12/14

Now that I have the communication sockets figured out, I should actually use it to allow tabs to run in processes.
Since I am pretty much making a window manager, I will use the term `app` from now.
There should be a one to one mapping between an instance of an app and a connection.

The singularity app protocol (sap ?) should support extensible features.
What I mean is that, if the server and client both want to support dragging as an optional feature, then they should be able to send dragging packets to each other.
But, if either of them don't know dragging, then dragging will not work, but the app should still work.

I can't think of a way to make this work well at runtime or to ensure complete safety, but here is my idea:

- Each feature (including the standard features) can be a crate or something
  - Has a unique feature id of a constant size (like u64)
  - Should have exactly one type each for `Event` and `Request`
    - These should all be serializable to and deserializable from binary (`&[u8]`)
    - These are the packets currently inside `singularity_common::tab::packets`
    - I am grouping `Query` into `Request`, but idk how to do type safe responses right now.
  - I think there is an error thing like this
- If a message with binary b of feature id f is to be sent, then send the tuple (f, b).

I was talking to a friend (@glolichen), and they suggested sharing a file that lists all the features that will be used.
Maybe I could send an initial message just to say what features each side supports.

2024/12/31

OK so kind of liking off on singularity if you couldn't tell but I'd stuff up but I'm gonna work on singularity now hopefully. Enter line
But anyways it's like 2 AM and my sleep is all messed up anyways so it's actually pretty early for me like I've been sleeping much later than this but anyways, I'm trying to sleep early today because I have to wake up early tomorrow, but I couldn't sleep just keeping this in my head because I was thinking about singularity instead of trying to sleep.
So I decided I'm just gonna write down the idea I had using voice memo on my phone for like voice dictation. You know what I mean
so the idea was anyways I've been thinking about I've been thinking about how to actually do like responses and stuff because it's kind of complicated to do it especially with multiple threads and now it's even multiple processes and before I go over my my current idea, I'm gonna try to go over the other ideas I had first so the first idea was not the first idea, but one of the ideas was to just create a new connection each time so if I wanted to get a response on like that, the overall window size in pixels then I would like somehow create a new response or a new new like form of process communication like maybe I could share some memory just to represent the response for that and then once that response, it's actually fulfilled then it would be useless and that is I think that would be the safest way so far but it's quite evident why that is not the best idea and the other way I was thinking was I was looking at how Wayland does just process communication between the client which are like the windows and the server which would be your your Wayland compositor and as looking at Wikipedia and the dogs and I actually also ask ChatGPT cause I couldn't really find a straight answer in the in the official documentation. I'm sure it's in there somewhere but I asked just how does Wayland manage like requests from the client that are expecting responses in my in singularity I'm gonna call these types of and chat. GPT told me I'm not sure if it's accurate but it makes sense so that's all I really need what it told me was that every request would have a unique ID associated with it and then for every every query would have ID associated with it and the response would somehow include the query ID in it so that we know exactly what the response is a response to. So and then.
OK, the app decided to crash on me, but I'll try to pick up where I left off. I was talking about the ChatGPT solution so it would have a unique ID for each request and then the response would somehow represent that it would somehow include that request ID and ChatGPT mention that it also included time data but that not like fundamentally necessary so I'm gonna ignore that for now and everything else that I've already implemented like the the type ID so the ID for actually like what kind of thing this is as well as the as well as the message length ID are actually things in Wayland already I kind of expected the type ID but I was actually surprised that the message length ID was something well ended. Maybe I copied it and then forgot about it but anyways the request ID seems like a not horrible ID for it but the only the only problem I have right now with that would be that there's gonna be multiple threats on the server side and I don't really care about the multiple threats on the client side that doesn't really matter but for example, what if the client sends a query but before the server sends the response, it sends just a event to the client so it's on the client receiver side. It's gonna get the event and then the response and then I doubt there's ways you could try to solve this, but it's all quite tricky and there there is bound to be a better way so I was thinking I could still do the response and request ID idea, but the only difference would be I would have a rapper for it so my idea for the rapper would be something like the universal client socket or something like that for the client and then for the server, it would be the universal server socket and I already have something called like universal packet socket or something like that or like universal packet writer I think is what it's called but then I could have inside a singularity universe so like client connection or client socket and that would either be a generic... Either be a struck with a generic or better yet I think having having it be a macro might actually be better then
I think I cut out again, but the big idea was just to have either struct with the generic or of a macro for a universal client socket and a universal server socket. This would work as a rapper around the around like even the unpacking ideally, but especially the ID for responses. One of the things it could do would be if you call like socket, query, blah blah blah then that would actually run a loop and then every time it receives any packet it would first check if it's the correct type of response and if it is the correct response then great that's like the basic case. We just return the packet response packet directly, but if it isn't, then maybe we could have it in the the message queue or something and then and then every time we get something from the server that isn't what we're actively looking for then we just put it in the queue and then we keep listening until we get what we're looking for and then later on, we can pass on the the socket or whatever to maybe like the main loop and then in the main loop it would just listen to all of the events that happened and it would start from the message Q and then it would listen if there was anything else and yeah I'm pretty sure that's already implemented for for sockets but from what I know, I don't know any way of directly modifying and accessing the message to that however you sockets does it so I guess we would just have a system of having two cues but that's really fine. I don't really care about the performance right now for something so trivial as that and the other idea I had with the universal socket macro was that I could have it so that we list all of the all of the event types somehow and then it would generate from that its own like Eno for that and maybe it could be called like universal events or something like that and universal events isn't actually defined in singularity because universal events could change depending on the events that the clients and servers want to support so I'll just be inside a macro so that it could change and actually the good thing about having a macro would be that since we're having it we're gonna have to manually serialize and deserialize it so it's fine to actually not have the the type itself defined in a common place cause the universal client and server sockets are going to be macro so they're not actually gonna define anything but it's gonna be like a blueprint for how to define the types and that's fine because we're gonna be manually serializing in deer realizing
i'm starting to feel sleepy now, which is good. I should be sleeping right now but before I end my ramblings, I just had one more technical thing I wanted to say so I could get it out of my mind and that actually has to do with the implementation so I was planning for the universal rapper to be not run on a separate thread even though that actually wouldn't be a horrible idea I think just having universal rapper an actual I guess instance isn't really the word I'm looking for, but but like yeah, having an instance of a universal rapper just simply be an object and not be like in its own separate thread, and then you can have a bunch of connections to the universal rapper thread cause that kinda complicates everything again and also I feel like having a separate thread is gonna be a pretty significant cost in a way like I feel like if I was writing a client and I had to end I didn't even have the choice of not creating a separate thread then that would be a pretty big turn off for me, but I would have the actual universal rapper just be an object which stores maybe a message Q and then the actual connection to the UNIX socket itself and in terms of the rust borrow checker, I think for each connection there should only be one like one actual object of the universal rapper, but I think having it work with references will be would be useful and maybe I was thinking if you only have a reference to the universal rapper, the only thing you can do is call request and the reason why I thought good idea is it because I feel like the events itself should be centralized like it would be kind of weird if like a sub component of the client was listening to the event, but I don't know. I'm just saying this because I want to get everything in my head out of my mind to just clear any thoughts that might Lynge and get in the way of my sleep

2024/12/31 (next day)

Wow... that is a lot of yap that I am not going to read.

I think I remember most of it so its fine.
Just make a universal client/server socket wrapper handler (name in progress) macro.
I am thinking something like:

```rust
use singularity_standard::{KeyboardEvent, MouseEvent, WindowRequest}; // this could be merged as StandardEvent, but I don't actually know if that is better
use drag_crate::DragEvent;
use interapp_communication::IACRequest;

generate_universal_client_socket!{
    events: [KeyboardEvent, MouseEvent, DragEvent],
    requests: [WindowRequest, IACRequest],
    // this would somehow have information on the corresponding responses for each branch
    queries: [StandardQuery],
    // default would be Universal
    name_prefix: MyCool,
};
```

which would generate `MyCoolEvent`, `MyCoolRequest`, `MyCoolQuery`, `MyCoolResponse`, as well as `MyCoolClientSocket`.
The first four I just listed would just be enums.
The ClientSocket would have methods: `read_events`, `request`, and `query`.

I really like the python `*args` syntax, so I want something like:

```rust
generate_universal_client_socket!{
    // `*StandardEvent` would expand out to its enum branches, like `KeyboardEvent` and `MouseEvent`
    events: [*StandardEvent, DragEvent],
    requests: [WindowRequest, IACRequest],
    queries: [StandardQuery],
    name_prefix: MyCool,
};
```

I am not convinced that queries and responses are type safe yet.
Another syntax I was thinking about is:

```rust
queries: {
    StandardQuery: StandardResponse,
}
```

but that is even worse.
A relatively jank but safe way would be to just have a new method for each query, like: `query_window_size` and `query_clipboard`.
To avoid name collisions, I could eventually create a `StdQuerier` and `WhateverCrateNameQuerier` structs,
which are generated from a reference to the client socket and only have the ability to send queries under that crate.
So, you could call something like: `client_socket.std_querier().query_window_size()`.

The information for queries, requests, and their relations can be reduced to a set of named mappings from query data to response data.
Each element in the set can be represented like: Name: InputType -> OutputType.

...

I will now try to implement what I wrote inside of singularity macros.
By the way, I think I already mentioned this link, but I believe this is the most helpful proc macro resource: https://www.freecodecamp.org/news/procedural-macros-in-rust.

Trying to expand with `RUSTFLAGS='--cfg test' cargo expand --manifest-path singularity_common/Cargo.toml tests::macro_demo`
has a weird error, and it seems to be caused by the `RUSTFLAGS='--cfg test'` portion.

I got a little confused on the test targets, but it turns out, you are supposed to put the tests OUTSIDE of source, just like examples.
You should also put benches outside, and I didn't even realize benches were a thing.
This is the project layout guide: https://doc.rust-lang.org/cargo/guide/project-layout.html
Very useful, especially since I haven't been adhering to best organizational practices.

I remembered the thiserror crate, which I haven't used yet (I unfortunately don't do proper error handling).
This is an example from their [crates.io](https://crates.io/crates/thiserror):

```rust
#[derive(Error, Debug)]
pub enum MyError {
    Io(#[from] io::Error),
    Glob(#[from] globset::Error),
}
```

I think I can do something like this for events and requests (query and response are slightly harder).
I won't need the `#[from]` specification, because I would assume all event cases defined by the macro are already events themselves.
(if the client or server both made their own custom events, it would get messy even if they agreed)

By the way, if there are nested enums, (like MyEvent has ClipboardEvent has CopyEvent), then there would be multiple ids sent in a packet.
I could optimize later, maybe by comparing the lists of all supported id types on connection.

I am going to write a declarative macro for event combine, which was done manually in macro_demo's MyEvent.
I want the macro to look like:

```rust
combine_events!(MyEvent: [ClipboardEvent, DragEvent]);
```

...

actually implemented, and it looks like:

```rust
combine_events!(pub MyEvent => [ClipboardEvent, DragEvent], 9000);
```

the differences:
- I can add visibility qualifiers (an actual improvement)
- `:` to `=>`, just because declarative macros don't support `:`
- need to specify the event id
  - should talk about this

So, how should I procedurally assign the IDs?
The the current state of the macro requires the caller of the macro to generate the id themselves,
which is the safest way on my end, because if something goes wrong, then it is the user's fault.
But, I want to eventually automate this as well.
I will list ideas, and something to consider is how I am going to deal with versions:
- Just generate a random number non-deterministically
  - Would change at every compilation, I don't like this
- Generate a random number based on contents
  - Actually pretty good
  - Would change every version but is deterministic
  - Changing every version will preemptively catch possible inconsistencies, but will be finicky
- Generate based on the name
  - Would be same across versions, not sure if that is good
  - The pro case for this would be when an event adds more subevents
- Generate based on the name and the id's of the components
  - It would change id when the contents change, but not if the impls change (I thought this was the best idea, but now that I explained it, I think just the name might be the best)

For all the ideas, I should also include the name of the crate, so multiple crates could have distinct events with same names.

2024/1/1

Okay, I am on a new system with Fedora and Gnome (the one I've been working on and will continue to work mostly on was NixOS with KDE).
Luckily, getting it to run wasn't too bad, I just had to run `nix --extra-experimental-features nix-command --extra-experimental-features flakes develop`
because I don't know where my Nix global config file is and I can't permenantly enable nix commands and flakes.
And when it ran with the dependencies, it ran smoother than what I expected.
The display and mouse clicks worked, and the mouse clicks affected the display in the expected ways.
Unfortunately, the keyboard input just had no effect on the app.
For now, I will ignore this problem (sounds like something I will regret later),
because I just need to set up the infastructure for IPC, which doesn't need me to run until much later.

Anyways, the previous thing I did was make the `combine_events` macro.
Next up: combine requests.
After that, I think I can implement a 

Requests should be so similar to events (I'm pretty much just reimplementing rust enums),
that I have a very slight urge to create a meta macro that would be like:
`meta_combine!(Event, Request)` which would define the `Event` and `Request` traits,
and define the `combine_events` and `combine_requests` macro.
But, I won't do this.
Or, I could make everything under just the packet trait (not a bad idea, actually). 

Sidenote: I was trying to think of a better name than `combine`,
and remembered union types.
I looked it up, and it turns out rust actually does have a union keyword.
It seems kinda cringe though, ngl.
Regardless, I think `event_union` is a better name for the macro than `combine_events`.
