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

...

It turns out I can't develop until I give smithay its dependencies,
because VSCode rust analyzer breaks right now.
I can run the dev flake in CLI, but I don't know how to enable it for all of vscode.
On my nix system, I used the nix env and direnv extensions,
but I can't enable flakes permenantly, so the automatic nix env breaks.
I might be able to get it working with [Docker](https://docs.docker.com/engine/install/fedora/).
VSCode has a [guide](https://code.visualstudio.com/docs/devcontainers/containers) on
dev containers with Docker.

That seemed like a hassle, so I just ran: `nix-env -iA nixpkgs.fontconfig`.

I actually hate nix now.
I tried to like it, but reproducible on any system is complete dog cheeks in practice.
I'm going to try to figure out how to do this without nix at all.

https://packages.fedoraproject.org/pkgs/rust-smithay-client-toolkit/ is a thing.
Running `sudo dnf install rust-smithay-client-toolkit` says it isn't a package though.

Okay, I just installed a bunch of dnf packages until it worked.
For future reference, some of them are:
- fontconfig
- fontconfig-devel
- libxkbcommon
- libxkbcommon-devel

...

I abstracted packets to not care about Event vs Request.
Queries and Responses might need to be slightly different though.

...

I don't think I need to use macros for universal client and server.

2025/1/2

At least for just events and requests, I can just do `UniversalClientSocket<Event, Request>` and same with server.

...

I think a derive macro would actually be better for making things into packets.
It is more flexible and would allow for other derives as well.

2025/1/3

I got communication between server and client on the same thread.
Next up (in no particular order):
- Set up testing for multiple processes
  - Proper testing? Yuck! I'm not going to do that until I absolutely need to
- Do Query and Response
  - Need a macro
- Set up the unix socket stuff inside of singularity_common
- Improve the current derive `PacketUnion` macro
- Implement into singularity
- Organize properly

I'm going to try setting up query and response.

I want to add modularity, so that its not all just flat query and response pairs.
I will try to come up with a basic example of usage (ignore implementation, because implementation is trivial with respect to usage):


```rust
let my_query_bundle: MyQueryBundle = todo!();
let addition_response: AdditionQuery::ResponseType = my_query_bundle.get_math_query_bundle().query_addition(socket, AdditionQuery(1, 2));
```

so from the object `my_query_bundle`, we would get the sub-bundle of type `MathQueryBundle`,
which would contain the query: `AdditionQuery`.

I think I have an idea from this.
The most basic type would be a `QueryPair`, which consists of a query type and a pair type.
Then, there is the `QueryBundle`, which is a collection of sub `QueryBundle`s and `QueryPair`s.
All `QueryPair`s have a default `QueryBundle` consisting of just that pair.

Something like: `universal_client_socket.query(AdditionQuery(1, 2))`,
would not reaveal the type of the response...
Actually, it could.

Holy guacamolie, I think I just talked myself into a brain blast!

```rust
pub trait UniversalQuerier {
    fn query<Q: UniversalQuery>(query: Q) -> Q::ResponseType;
}
impl UniversalQuerier for UniversalClientSocket {
    todo!()
}

// UniversalPacket is the one with the do and from data
// as well as the packet id
// It is currently called `PacketTrait`

pub trait UniversalQuery: UniversalPacket {
    type ResponseType: UniversalPacket;
}
```

I think I was previously overcomplicating this whole thing.
With this way, query and response should actually be easier than event and request.

On the server-side, I was thinking of handling responses in the same way
as I have done with the mpsc channels:
`client_handler.respond(move |query| { ... })`.

This approach is acceptable, but I think I need to just assume
that the server will not return the wrong query type.

With that, I will assume that the only packets are:
queries from client to server and responses from server to client.

Query raw data will contain:
- Packet length
  - already handled by the byte writer and reader
  - const size
- Packet type
  - To say that it is a query and not a request
  - (I could eliminate this by just saying everything is a query)
  - For all queries, this should be a constant value
- Query instance id
  - Unique to each instance of a query (eg, even if you query size multiple times, each query will have a different instance id)
  - const size like u64
- Query type id
  - This actually says what type of query the query is
  - This would distinguish between things like: QueryName vs QuerySize
  - const size like u64
- Query inner data (Optional)
  - This would be defined by the query type

It might make more hierarchical sense to put the query type id
before the instance id, but I think practically it makes more
sense to do instance id first, because instance id should be read
even if the query type id is unknown.
(it really doesn't matter, even though my reasoning is kind of bad)
Oh, another reason is that if I did have query bundles,
and stored query type hierarchically in the raw data,
then it would be better to have the instance id first
(even though storing hierarchy would be inefficient).

Response raw data will contain:
- Packet length
  - already handled by the byte writer and reader
  - const size
- Packet type
  - To say that it is a response
  - For all requests, this should be a constant value
- Prompt Query instance id
  - Would be the same as the query instance id that prompted this response
- Response type id
  - This should be easy to tell from the query, so I am considering just not having this
  - Will have a special id reserved for unknown queries
  - If the response has a wrong type id that isn't the null id, then panicing will be understandable
  - const size like u64
- Response inner data (Optional)
  - This would be defined by the response type

For unknown queries, the server should just give a response with
a special `Null` response type id (probably like 0).
I considered having a `NullResponse` packet type, a `Null` response type id,
no response type id and inner data on the null response,
or just having an additional boolean represent
whether the response is null or not.

I like this system (of flat pairs instead of tree organization) a lot,
that I might actually change the events and requests to this way.
I don't like it for the fact that it is structured in a flat way,
but it does seem to be the simplest implementation.
Actually, I don't like it that much (I am an indecisive person).
I think that sending should allow for flat sending to avoid boilerplate,
but recieving should allow for a hierarchical structure.
Maybe I am in denial because I can't figure out a way to get hierarchy
for queries.

In the end, I should stop caring the specific implementation.
I just want to make reasonable progress in a reasonable time.

Okay, I am going to start.

...

2025/1/10

I realize that the standard is actually little endian, not big endian.
I will change that later for everything.

I think I should also write a new manifesto for singularity,
since a lot of things have changed.
The summary will be that I am trying to abstract things like UI so that
everything can be organized by a central organizer and apps can work nicely
with each other.
Right now, there are different OS's which run apps and those apps have their
own ways of organizing their UI components.
With a browser, it is clearest to see how many layers of different protocols
there are.
This limits customizability on the user side
(eg: no standard way to set color pallete,
the closest thing to this is dark vs light which is just 2 options
and apps need to go out of their way to support it;
eg2: standard setting and shortcut management)
and prevents compatibility between apps.
HTML kind of does this, but it is bad.

...

There is something called `TypeId` in `std::any`,
which might allow me to do the id stuff by default.

...

Adding onto the philosophy, I want to be able to remove the sidebar and line counter from my text editor
very quickly, like doing inspect element and then save those settings into a view template.

2025/01/15

I have been working on other stuff, but today I worked on tests for the query response sandbox.
I should have committed before, most of the changes in this commit will be from a few days ago.
I have often worked on a pretty big change, almost gotten it finished, and then worked on other things for a few days before putting the finishing touch and committing later.
Ideally, I would be using github issues more often, but it seems unnecessary right now.
I will change my policy to just commit broken code at the end of the day if I know I will be working on other stuff for a few days.

Also, I looked into Unix Domain Sockets and TCP sockets, and in TCP, it seems like they know what response corresponds
to what request, because responses are ordered the same ways as requests.
I think I like the Uuid way more, even though it has slightly more overhead.

Erhm, I made thread_test to generate the minimal reproducible example, but it worked and then the simplified version of the original code started working too.
I will commit the two minimized versions before it doesn't work again.

...

NOOO, I committed, and ran without chaning ANYTHING.
It didn't work.
After further teseting, it seems to be non-deterministic because threads.

Okay, I fixed it (well, I ran it ten times in a row and it worked) and now I know why.
I had to do the listener creation before starting both threads, to ensure that the client wouldn't ask to connect on a socket before it even was created.
The fact that the error was non-deterministic actually led me to try this fix.

...

By the way, adding the `--show-output` flag to `cargo test` does show output, but only after the test ends.
I had to add `--nocapture` to debug why it doesn't even end.

...

I got the sandbox to work, so here are next steps:

- [x] Move unix domain socket helpers to `singularity_common`
- [x] Abstraction for bytes unwrapping/splitting
- [x] Make a new all packets sandbox, to merge the query response sandbox and the event and request code from sap
  - Probably should split the roles by packet type (eg, the clientside sap interface should have two different types of things for sending queries vs requests and two more different modules for parsing events vs responses). I guess this is pretty obvious actually.
- [x] Move the stuff in query response sandbox into sap
- [ ] Change all big endian to little endian

2025/01/16

It turns out query_response_sandbox sometimes non-deterministically fails,
but I feel like it will be fine.

2025/01/17

I have an abstraction idea that might be really bad,
so I won't implement it, but I wanted to write it down
for the sake of ...whatever.

I think I can further abstract event and response by just having
a general packet thing.

I think I am going slightly mad right now.
The difference is between defining a basic structure for packets and
providing the actual parsing, versus just letting them define the parsing stuff themselves.

...

I just realized that the server side connection doesn't even need a queue,
because I only needed queue because client might ask for a specific request.
I will fix this after committing though.

I think I figured out a way to jank the type system.
First, use the `type Query` instead of `<Query>`.
Then, I think I can make helper methods inside of the query responder,
which I believe will let me avoid problems with unknown size errors I would
have gotten if I put the helper functions there.
Additionally, I can make an InnerQueryRespond to prevent the helper functions
from being overridden and to prevent them from clogging the user's list of usable methods.

The actual calculations that need to be done when sending back a query response, given that we know it is a query:

1. Split `packet data` -> (`query instance id`, `query type id`, `query inner data`)
2. Try to match `query type id` to an actual `query type`
   1. In our case, we need to find the `query responder` with this `query type`
3. Generate `query object` of the correct `query type` from the `query inner data`
4. Generate the `response object` by asking the correct `query responder`
5. Generate `response inner data` from `response object`
6. Generate `response data` by combing `query instance id`, `response type id`, and `response inner data`

I think steps 2-5 should be in the trait helper functions,
because they require information about the type that I am not sure how to get from `&mut dyn QueryResponder<Query = dyn Any>`.

Uhh, the trait stuff is stricter than I thought.
You know what, that actually sounds like a problem for future me.

---

2025/01/18

Gosh, I wish I could take credit for this, but I asked chat gpt to help me debug or brainstorm a new way,
and it actually thought of a good solution.
It is like learning that CRANE (technically SALET, whatever) is the best worlde starter;
you don't want to use it because it is not your solution, but you kind of have to because it is the best.
The important section is this:

```rust
pub trait TypeErasedResponder {
    fn respond_erased(&mut self, query_data: &[u8], query_instance_id: QueryInstanceId) -> Option<Vec<u8>>;
    fn get_query_type_id(&self) -> IdType;
}

impl<Q, R, T> TypeErasedResponder for T
where
    T: QueryResponder<Query = Q>,
    Q: UniversalQuery<ResponseType = R> + TryFromData + 'static,
    R: PacketTrait + 'static,
{
    ...
}
```

and I swear I only looked at the type hinting part of its code, because I don't want to just use generative AI
for singularity.
I just used it to get an idea and I will write the code myself.

But dang, I really don't know how to feel about all this AI stuff.
Like, I saw a video about a writer who was replaced by AI, and now I am just wondering if that is going to happen to me.
That, in addition to the oversaturation of CS people in general is quite frightening.

At least I can look at the current state of Devin and know that AI won't outperform humans yet,
but it will only grow smarter.

I'm not sure how the ethics of all this works either, because I suspect that Chat GPT was trained on lots of data
without permission, and I kind of feel like I am stealing from those people, but my current position on the matter
is that consulting Chat GPT is somewhere between a rubber duck on steroids and just looking at someone else's code.
I believe that pretending like generated code is someone else's code is a pretty safe view (less likely to be
intellectual theft), so I will just ask it for ideas and help debugging.

To my credit, I was already trying to do something similar to this in the `InnerQueryResponder`.
The big difference is that I just had to take in the argument of type `Vec<&mut dyn InnerQueryResponder>` to begin with.
I actually feel like I could have thought of this if I gave myself a few more days,
but I didn't and I was able to save those hours so I can make more progress, so I can't complain.

...

I will actually implement the rest of the logic for `handle_incoming` now.

TODO:
I am getting an annoying clippy warning about `InnerQueryResponder` being more private
than the `handle_incoming` function,
and I still want to keep it private to prevent tampering,
but I will consider fixing it by having yet another trait, if that is possible.

...

I couldn't do that easily, so I just made it public with a sign that asked the user not to override it.

---

2025/01/19

Holy guacamole, I might be the GOAT of all time of all time.

First time running, and no errors!

Put in that meme of kronk going "Yeah, its all coming together"
because it all really did just come together.

...

Okay, so despite that MASSIVE W, I still need to add more testing,
which I didn't do because I was scared it wouldn't work.

I think the single process tests work, I will commit this then test on different processes,
then put the `all_packets_sandbox` into `singularity_common`.

...

Today might be my day, multi process worked smoothly as well.
I will commit then move it to the actual libraries.

...

The little warnings were driving me crazy, so I got rid of them.

---

2025/01/19

There are still things to improve with `sap`, but I think I can start incorporating it
into singularity now.

I think I should give a high level overview of the sub-projects for singularity:

- Singularity Application Protocol (SAP)
  - It describes a way to send extensible packets between any sap supporting server and a sap supporting client
  - Sap is like the wayland protocol, the sap server would be your desktop environment, and the client would be any wayland app
  - For now, each SAP connection corresponds to exactly one window
- Singularity Project Manager
  - A way of organizing projects so that projects can talk to each other
  - This will be used by the singularity tab manager
  - Primary purpose is to neatly organize how persistent data related to singularity is stored
- Singularity Tab Manager
  - Possible name: Stabor (singularity tab organizer)
  - This is the official SAP server, the only one that I will be working on (probably)
  - Its job is to manage and provide tools for sap clients (tabs)
    - Handle compositing and UI
    - Relay communication between tabs
    - Organize tabs
- Tabs
  - Each tab is a SAP client
  - Example Tabs:
    - Chro
      - The time manager
      - I might make this a seperate thing, and have it support terminal or SAP Gui
    - Terminal (Sterm ?)
      - (realistically, once I have terminal, I unlock most apps I need)

I will start deprecating the old methods of communication and integrating sap into the singularity tab manager now.

As expected, there is a lot of things I need to rethink.

2025/01/23

Okay, I didn't commit like I planned, and you can tell from the log dates that its been a few days.

Well, I guess I will think of everything before committing since I am late anyways.

I will split all my modules into these crates:

- Singularity Common/Utils
  - Datastructures and stuff that are used by many things
- Singularity Macros
  - Should be in `singularity_common`; this would ideally just be a module in common but proc macros currently need their own crates
- Singularity UI
  - Abstracts all the Backend specific stuff to provide the bare minimum UI support
  - Currently just supports wayland
  - No change needed
- Singularity Project Organizer
  - (SPORG?)
  - Not sure about this name
  - Used to be in `singularity_common`
- Singularity SAP
  - Yes, the name is redundant, whatever
  - Used to be in `singularity_common`
- Singularity STABOR/SDE
  - Alternative name: Singularity Desktop Environment
  - This is the canonical implementation of the SAP server to handle tabs
  - This used to be just `singularity_manager`
- All the tabs can be in their own crates or something

Something I want is extensible UI widgets with shared libraries,
and this idea can be extended to things outside of UI.
There are UI primitives. For the sake of example,
lets just say that the only UI primitive is the pixel grid.
The set of all primitives is already agreed upon,
and must be standardized.
But, suppose an app wanted to display text.
Without widgets, the app would have to draw the text
itself onto a pixel grid.
This is bad for a few reasons.
First, this lacks standardization.
If there were multiple apps, that had to do it themselves,
then all of them would have different fonts and it would be ugly.
Secondly, due to the lack of standardization,
user side customization would be difficult.
Also, this could add performance overhead.

The solution would be to have shared widgets.
Each tab can return some composite of widget and primitives.
If some widget protocol is manually implemented by the user's
display environment, then that implementation is used.
But, when a tab uses a widget protocol, it must also define
some shared library type thing that would handle the default
case.

...

I think I might just rewrite most of sporg and the tabs.

2025/01/30 12:52AM

(I am writing this entry to talk about talking to someone about singularity.
The other changes are things I've been working on and is unrelated to this.
Ik, I said I will commit more but too late.)

I talked to someone who has a lot of experience.
Other than the one friend I talk to (@glolichen), this person is
kind of the first person I've talked to about singularity in a pretty deep level.
They didn't care about the actual code, but they asked a lot about the idea itself.
We talked over email for a few days, and today (technically 2025/01/29 7-8PM),
we talked over phone.

I really have to sleep so I'll keep this short.
If it really matters, I will put in the email logs later, so I will just summarize the call.

The main question/advice was to just make an actual window manager.
The pros are performance, simplicity, and the fact that I can simply use any app that already exists for wayland (or x11 if I make a x11 wm).
But, the inter-app stuff wouldn't be a part of the wm.
They mentioned `ocl` for the office suite.
They also said that Xserver was a program on its own, and I should experiment by running bare xserver from terminal then running apps manually from the cli as well,
like xclock.

I suggested, for the inter app stuff, if I were to make a window manager, I could have a special app to handle all
the inter app stuff.
But, I didn't really explain my use-case fully.

I am not completely sold on the idea of making a window manager yet, for a few reasons.
The first is the inter-app stuf.
Secondly, they said making a text editor and terminal from scratch is impossible for just one person.
But, I think I can embed some other open source terminal for singularity at the worse case and run vim on it
(but at that point, I could have just made a terminal session manager).
I will do research on that though:
[this reddit thread](https://www.reddit.com/r/rust/comments/1d47bl1/suggestions_on_a_gui_framework_for_embedding_a/)
says I can embed alacritty.
It suggests using [alacritty embeds](https://docs.rs/alacritty_terminal/latest/alacritty_terminal/#reexports).
It gives [this example](https://fuchsia.googlesource.com/fuchsia/+/refs/heads/main/src/ui/bin/terminal/).

I will have to think about this for a while.

2025/01/30 1:35AM

Roadmap to recovering from restructuring:
- [x] Implement the TODO's in singularity macros
  - [x] Test by making a print_test_server in sde and print_testor in standard tabs as a bin, where standard packets are sent and printed on both ends
- [x] Add basic `standard_packets`
  - [x] TODO: bare minimimum packets for standard packets
- [ ] Figure out how to do the cfg feature stuff (might already be working, if so, just verify it is working)
- [ ] Start the actual sde, that just displays the UI, no organization
- [ ] Implement some basic thing in std tabs, like the worst cookie clicker ever
- [ ] Add organization code back into it
- Figure out the rest (eg: adding sporg into it)

2025/01/30 7:34 PM

I posted a [question on macro expansion](https://users.rust-lang.org/t/expanding-inner-macros/124887) to the rust lang forum
because I realized I needed to learn to ask for help.
There is a chance that no one answers, but that is fine.
I had to create a simpler version of my question, but if that simpler version is answered,
I should be able to jjust apply that solution to my actual macros.
I will just commit now to log this.

2025/02/01

I got two responses so far.
Feels unusual that Steffahn replied, because I've seen their replies on a lot of
the rust threads in the past, and I kind of assumed they were a celebrity or
something.
But it does make sense that the people who comment the most would be the ones to
comment on mine.

The responses said that it isn't really possible.
But, there is a [nightly feature](https://users.rust-lang.org/t/expanding-inner-macros/124887/5)
which should hopefully make it possible eventually.

---

I am adding the basic `standard_packets`, and I am not sure how to do `SpawnChildTab` because
I have to represent the idea of a generic tab.
This is definitely an important feature, but I don't think it should be a standard feature,
so I will just have it not be one.
A related feature would be `SpawnDefaultEditor`, `SpawnDefaultBrowser`, and etc
(or I could have `DefaultEditorQuery`, and etc).
Well, now that tabs are processes, I could represent a generic tab with the path to run the process
along with the arguments, much like `Command`.

Also, I think the display should be done with shared memory but the problem is that I don't know how to do that in rust.

2025-02-11

I felt pretty stuck (both in this project and in life, ha ha ha ha...),
but I am working on changing how the derive macros work.
There are now 3 different derive macros:
- `Packet`
  - Impl's `PacketTrait`, but assumes `Datable` is implemented elsewhere
  - Mainly just generates the `PACKET_TYPE_ID`
- `PacketUnion`
  - To be used on an enum where each variant corresponds to a unique `PacketTrait`
  - Impl's `Datable` (`ToData` and `TryFromData`)
- `Datable`
  - Impl's `Datable` for an enum or a struct composed of `Datable`'s

The change that isn't obvious but should be mentioned is Datable impl for enums.
The previous impl is now packet union, and used the packet type id to differentiate
between variants.
The new impl for this is to define the numbers corresponding to variants in the macro.

---

I implemented Datable for a bunch of classic types, and also added proper unit tests
for them.

I am also implementing Datable for `singularity_ui` elements,
and I've just been copy and pasting the types into another file that can access the Datable macro
and singularity_sap, then using the derive, then expanding,
copying the expanded output, and pasting it into byte_stream.
I am doing all this because rust doesn't allow circular imports easily,
and I also can only use derive macros at teh struct/enum definition.
Maybe there is a way to automate this in `build.rs`,
but for this scale, that would be more work.

---

I implemented this way recursively, until `UIElement`.
I don't have the UI Events yet, but I can now get started with some very basic communication.

I was going to make a boring tab called `UIElementDemo`,
but life is too short to do boring stuff.
I am going to make a fortune teller tab,
which is functionally just a wallpaper, but I want it to be something fun.
It is going to be like one of those quirky `.bashrc` settings.
My friend pipes fortune into cowsay, and I think that is cool.

I'll just have the fortune teller display the time and a fortune.

...

I have to think about how to represent tabs in singularity.

2025/02/15

I tried to get either nvim or emacs working,
and I really want to get into emacs,
but it has hour-long videos of just the basic config,
and when I combine that with the fact that I use NixOS,
I think I have a better chance of just finishing singularity
than setting up Emacs.
People joke about not being able to exit vim,
but I can't even set up Emacs.
(I know Doom emacs and spacemacs is a thing,
but they work even less with NixOS.)

Anyways, let me get back to singularity.
The problem from before was to do with representing tabs.

I will outsource the hard part of this decision to my future self,
and for now, for the most basic purposes, I just need it to be
a connection to the client.
I will put this inside `singularity_sde`,
even though this should belong in a singularity server toolkit.

I think I can just reuse what I deleted in the restructuring,
and the most up-to-date stuff for that is in:
https://github.com/mathkimchi/singularity/tree/8e8e02348cb41f0d077783aaa2ca7d3d69288d22/singularity_common/src/tab.

The hard part will be to somehow allow representing this in data
for the following features:

- Preserving tabs in between sessions
- Spawning Children Tabs

Now, the obvious way would be to store the commands that
spawn the processes that spawn these.
But, this idea on its own doesn't address the problem of
matching tabs to actual processes.

An example scenario of this would be:
When the SDE spawns a process and expects the process
to request a tab and the SDE wants to put that tab
somewhere in the hierarchy.
This is the case for both restoring the tabs from a past session
(want to put each tab in the past session's hierarchy),
and for spawning a child tab
(want to put the child tab as a hierarchical child of parent).

Also, it would be difficult to even get this for new tabs,
so it is hard to even start.

I can think of more than one way of resolving this:
Firstly, just have the tabs give this information
to the SDE.
This could be like a normal request, or as a special request
that is sent on connection start.
Now I will explain the radical solution:
Alternatively, rest the "burden of initiative"
on the server.

I explained it in the car with a voice recording,
and I am too lazy to transcribe it all.
I won't clutter the repo with the sound files,
but on my phone, it is saved as:
`Hobey Backer Memorial Ice Rink`,
`Princeton University 63`,
and `Princeton University 81`.
(I am not sure why it skipped from 63 to 81.
The phone was weird and kept stopping,
maybe because the audio was connected to the car.)

But, I am not sure about this idea.
If there was a "singularity way" of doing things,
then I think this idea would be 100% the singularity way.
I can't explain why, but just this non-conforming,
complicated, unnecessary challange of standard practice is precisely
the type of thing an idiot like me would enjoy.
Plus, "burden of initiative" is a cool term,
so I guess I just forced myself into doing this.

In all seriousness, I think a change of this magnitude,
at this stage (when changes and commits are already slow),
requires careful consideration.

I am reminded of local web services like openwebui.
But, to my knowledge, services mean that they run in the background,
and I don't want to do that.

I think I could have something similar that caters better to my wants
with shared libraries.

2025-02-17

I don't think I will be doing the webpage-like way.
but I still want to support many of the features like
the redirect pages system.
but I think the local storage should suffice for reopening tabs.

I think piping stdio could actually be a not bad solution,
and looking back at a [comparison of ipc methods](https://3tilley.github.io/posts/simple-ipc-ping-pong/#approach-1-pipes),
I realized piping is actually faster than TCP/UDP sockets,
and [a similar article](https://www.baeldung.com/linux/ipc-performance-comparison)
shows compares pipes with Unix socket specifically, and says it is 30% faster
(this much of a difference doesn't really matter, but knowing that a new approach
is a speed upgrade makes me feel good about implementing it).
Also, I think the piping is similar enough to unix sockets such that it wouldn't
be difficult to support both.
Piping will allow for SDE side initialization, but not the more webpagey features
like redirecting.

2025/02/18

Okay, so, I decided I will implement just Unix Sockets for now,
but in serialization, represent tabs as commands.

...

Actually, I am trying to fix the code I previously wrote,
and a lot of the past code expects the tab to be initialized by the sde.

2025/02/19

Stdin doesn't have a vanilla way of polling,
so I made a wrapper for `Read` using mpsc.

The downside of the piping is that my main tool for debugging just disappeared.
Ways to debug would be to create a request, or to write to files.

I am going to change the old idea of `tab_type` to `tab_command`.

2025/02/21

I implemented datable for the UI Event stuff,
so now I think I am almost done with migrating to multiprocess.

As a reference, this is from 2025/01/30 (has been updated as I went)
Roadmap to recovering from restructuring:
- [x] Implement the TODO's in singularity macros
  - [x] Test by making a print_test_server in sde and print_testor in standard tabs as a bin, where standard packets are sent and printed on both ends
- [x] Add basic `standard_packets`
  - [x] TODO: bare minimimum packets for standard packets
- ~~[ ] Figure out how to do the cfg feature stuff (might already be working, if so, just verify it is working)~~
- [x] Start the actual sde, that just displays the UI, no organization
- [x] Implement some basic thing in std tabs, like the worst cookie clicker ever
- ~~[ ] Add organization code back into it~~
- Figure out the rest (eg: adding sporg into it)

I don't know what I meant by the cfg stuff,
I assume it is working.

I ended up never having to remove the organization code,
because I was able to reuse 90% of my old SDE.

I know what I still need to do, but right now, I think
I will commit this, and merge this branch into dev,
marking [#3](https://github.com/mathkimchi/singularity/issues/3) as completed!
(I should have made smaller issues, but whatever)

## [#6](https://github.com/mathkimchi/singularity/issues/6)

I made a new issue (for future reference, clicking the create branch with GitHub desktop is probably better than local).
You can see it in the second heading title for this section.
From now on, I think this will be the formatting of my devlogs.

I am going to strive away from the object orientedy way I previously handled this,
meaning the structs will have data but the bulk of the logic will be outside the struct.
Actually, I'll put the file manager specific stuff along with the struct (as an impl),
and have the more tab-related boilerplate stuff (client stream) outside.

This actually requires session data, so I will need to think about that as well.
I will start off by using a hard coded path.

...

I got displaying to work, but suddenly, none of the keyboard inputs are working,
even for shortcuts outside the tab like `Ctrl+Q` and tab traversing.
The old commit isn't working either.

Okay, it was because NUMLOCK was interfering with the shortcuts.
I really gotta do something about shortcuts.
You know what, I am going to make that a new issue.

...

2025/02/22

I am implementing Editor, which previously used the components.
I will add just the textbox component, but I am getting import conflicts by having it in singularity common,
so I will move this into a new rust package: `singularity_sttk` (singularity singularity tab toolkit)
which should be like the Smithay client toolkit.

...

Allowing file manager to spawn editor went pretty smoothly.
I think I can commit and PR this.

## [#11](https://github.com/mathkimchi/singularity/issues/11)

2025/03/03 2:30 AM

I am working on the big file for the SDE (the `singularity_sde/src/project_manager/mod.rs`),
and autoformat simply does not work on this thing.
I think it is caused by the large size of the file.
Handle input is the immediate biggest suspect, since it is long and has a lot of nests with all the shortcut cases.
Like I previously mentioned, I want some standardized method of doing shortcuts.
After this commit, I will actually start with that.

2025/03/03

I will make UI actions (more or less shortcuts).
I initially wanted to somehow make some UI actions impossible to represent from some modes,
but I think it is fine to have that.

Mode is just FSM, and UI actions are like events/events.
A [stack overflow post](https://stackoverflow.com/questions/35439546/a-pattern-for-finite-game-state-machine-in-rust-with-changing-behavior)
has a pretty good example, and it allows for incompatable events to be represented, which I guess

This [video](https://www.youtube.com/watch?v=KdLTqyblbo4) seems to use a similar example,
so I guess the pushing machine is a famous example or something.

In this example, I would want it so that the user wasn't even allowed to push on lock and coin on unlock
but it seems that they make it possible and just ignore.

I feel like I should make this a new github issue, but whatever.

...

Implemented the actions, crossing my fingers before running.

Oh yeah, it almost all worked first try.
I just had to change command palette from alt+shift+P to ctrl+shift+P.

...

Many improvements to be made for actions:
- Define actions in a seperate file to be read at runtime
  - Special syntax for shortcuts during specific mode, for shortcuts regardless of mode, special group of keys for tree traversal operations, etc, kind of like regex
- Somehow make UI actions more safe
  - Ex: If a ui action only happens in Mode::ChoosingFocus, then the UI Action can actually contain the choosing focus' data
- Have extensions define their own actions too

...

Okay, I don't want to dwell on actions, but I did make things slightly better by adding a common shortcut style case.
I didn't vigorously test the new change like I did with the Actions first time, but nothing seems to be broken.

...

2025/03/04 12:56AM

One more change before sleep: add display for command palette.

Oh man, I have a 8AM final tomorrow that I should have studied for...

But for good news, I am almost at the 10k lines of rust anniversary for Singularity!
(I swear I don't just constantly look at the word count; I do it less than one day per week.)

This is the total rust `wc.sh`:
- 9114 lines
- 26513 words
- 322802 characters

Counting the DEVLOG seperately, it is (*was, before this line):
- 2740 lines
- 24155 words
- 141563 characters

Progress today has been very fast.

Okay I'm going to sleep now.

---

2025/03/04

I have an idea for making invalid actions (transitions) unrepresentable.
https://hoverbear.org/blog/rust-state-machine-pattern/
goes over some methods, but I want to try something slightly different.
There are some insights from that article that I will attempt in my approach as well:

- Action object consumes state
- State variants get their own structs
  - Doesn't impact safety but improves clarity imo

I wanted to do something like:

```rust
let action = Action::from_ui_event(self.mode, ui_event);

self.mode = Self::next_state(action);
```

But even if I promise to put the value back, the [borrow checker doesn't let me](https://users.rust-lang.org/t/can-i-move-out-of-a-mutable-reference-as-long-as-i-promise-to-put-some-valid-data-back-in-before-i-use-it-again/108417/3).
[This crate](https://github.com/alecmocatta/replace_with) seems to take care of that,
but I don't want to use it.

I really don't like doing this, but I will swap the mode with a default, the `Mode::TabFocus`
and override it afterwards.

...

hmmm...
I don't like this at all.
I did things like:

```rust
Quit(Mode),
ChooseFocus(ChoosingFocusMode),
```

but I don't like how it actually turned out.
I'm actually just going to copy the current `mode.rs` into here:

```rust
use crate::tab::TabHandler;
use singularity_common::utils::tree::{
    id_tree::IdTree,
    tree_node_path::{TreeNodePath, TreeTraverseOperation, TREE_TRAVERSE_KEYS},
};
use singularity_ui::{
    display_units::DisplayArea,
    ui_event::{Key, KeyModifiers, KeyTrait, UIEvent},
};

// #[derive(Debug, Clone, Copy, Default)]
#[derive(Debug, Clone)]
pub struct TabFocusMode;

/// If ChoosingFocus, there should be a special window app focuser
#[derive(Debug, Clone)]
pub struct ChoosingFocusMode {
    focusing_index: TreeNodePath,
    plucked: Option<IdTree<TabHandler>>,
}

#[derive(Debug, Clone)]
pub struct CommandPaletteMode {
    /// The string that the user has typed so far.
    /// TODO: make this use Textbox component to support cursor and stuff without duplicate code
    command_buffer: String,
}

#[derive(Debug, Clone)]
pub enum Mode {
    /// Focused on some app
    TabFocus(TabFocusMode),
    /// If ChoosingFocus, there should be a special window app focuser
    ChoosingFocus(ChoosingFocusMode),
    CommandPalette(CommandPaletteMode),
}
impl Mode {
    // pub fn try_as_choosing_focus(&self) -> Option<(&TreeNodePath, &Option<IdTree<TabHandler>>)> {
    //     match self {
    //         Mode::ChoosingFocus {
    //             focusing_index,
    //             plucked,
    //         } => Some((focusing_index, plucked)),
    //         _ => None,
    //     }
    // }

    pub fn try_as_choosing_focus_mut(
        &mut self,
    ) -> Option<(&mut TreeNodePath, &mut Option<IdTree<TabHandler>>)> {
        match self {
            Mode::ChoosingFocus(ChoosingFocusMode {
                focusing_index,
                plucked,
            }) => Some((focusing_index, plucked)),
            _ => None,
        }
    }

    pub fn try_get_focusing_index(&self) -> Option<&TreeNodePath> {
        match self {
            Mode::ChoosingFocus(ChoosingFocusMode {
                focusing_index,
                plucked: _,
            }) => Some(focusing_index),
            _ => None,
        }
    }
}

/// Look at devlog 2025/03/03
pub enum UserAction {
    // SECTION - closing

    //
    /// Ctrl+q quits
    Quit(Mode),
    /// Ctrl+Shift+W recursively closes focused tab and children
    RecursivelyCloseFocusedTab(Mode),

    // SECTION - tab hierarchy operations

    //
    /// Alt+Enter from ChoosingFocus mode
    /// REVIEW: Rename to select?
    ChooseFocus(ChoosingFocusMode),
    /// Alt+Enter from NOT ChoosingFocus
    OpenFocusChooser(Mode),
    /// Alt+TreeTraverseKey should be like alt tab for Windows and Linux but tree based
    TraverseTabTree(Mode, TreeTraverseOperation),
    /// Alt+Windows+TreeTraverseKey swaps position of focused and what would be the new focused
    TreeSwapTraverse(Mode, TreeTraverseOperation),
    /// Alt+Windows+P
    /// I am fine with this technically being two different things to do but one action
    /// TODO: split this
    PluckPlace(Mode),
    /// Alt+Windows+Enter swaps actually focused and focusing
    TreeSwap(ChoosingFocusMode),

    // SECTION - command palette

    //
    /// Ctrl+Shift+P opens command palette (see: https://github.com/mathkimchi/singularity/issues/11)
    OpenCommandPalette(Mode),
    /// ESC quits command palette (see: https://github.com/mathkimchi/singularity/issues/11)
    QuitCommandPalette(CommandPaletteMode),

    // SECTION - tiling operations

    //
    // /// Alt+ArrowUp maximizes focused tab
    // MaximizeFocused,
    // /// Alt+ArrowDown minimizes focused tab
    // MinimizeFocused,
    // /// Logo+"=" (represents "+") increments title split
    // IncrementTileSplit,
    /// Logo+t transposes selected tile's container (hor<=>vertical)
    TransposeTileParent(TabFocusMode),
    /// Logo+s swaps selected tile's siblings
    SwapTileSiblings(TabFocusMode),

    // SECTION - misc

    //
    /// Key press isn't any of the keyboard actions; forward it to focused
    ForwardKeyPressTab(TabFocusMode, Key, KeyModifiers),
    /// Key press isn't any of the keyboard actions; forward it to command palette
    ForwardKeyPressCommandPalette(CommandPaletteMode, Key, KeyModifiers),
    /// currently, the only None case is when non-shortcut is performed on tab choosing mode
    /// REVIEW: is this good? just use option?
    NoAction(Mode),
    /// Resized. Currently ignore.
    WindowResized(Mode),
    /// Mouse press
    MousePress(Mode, [[u32; 2]; 2]),
}
impl UserAction {
    /// This is really to get around lack of if let in match
    ///
    /// For shortcut-like commands.
    fn handle_char_key_shortcut_presses(
        curr_mode: Mode,
        key_char: char,
        key_mods: KeyModifiers,
    ) -> Result<Self, Mode> {
        Ok(match (curr_mode, key_char, key_mods) {
            // Ctrl+Q
            (curr_mode, 'q', KeyModifiers::CTRL) => Self::Quit(curr_mode),
            (
                Mode::TabFocus(TabFocusMode) | Mode::ChoosingFocus { .. },
                'w',
                KeyModifiers::CTRL,
            ) => Self::RecursivelyCloseFocusedTab(curr_mode),

            // Alt+Enter from ChoosingFocus mode
            (Mode::ChoosingFocus(mode), '\n', KeyModifiers::ALT) => Self::ChooseFocus(mode),
            // Alt+Enter from NOT ChoosingFocus
            // NOTE: this pattern must be behind choosing focus
            (_, '\n', KeyModifiers::ALT) => Self::OpenFocusChooser(curr_mode),
            // Alt+TreeTraverseKey
            (_, key_char, KeyModifiers::ALT) if TREE_TRAVERSE_KEYS.contains(&key_char) => {
                // `' '` is a placeholder for some key that isn't in tree traverse
                // sad that match doesn't support if let syntax
                Self::TraverseTabTree(
                    curr_mode,
                    TreeTraverseOperation::from_char(key_char).unwrap(),
                )
            }
            // Alt+Windows+TreeTraverseKey swaps position of focused and what would be the new focused
            (
                Mode::TabFocus(TabFocusMode) | Mode::ChoosingFocus { .. },
                key_char,
                KeyModifiers {
                    ctrl: false,
                    alt: true,
                    shift: false,
                    caps_lock: false,
                    logo: true,
                    num_lock: _,
                },
            ) if TREE_TRAVERSE_KEYS.contains(&key_char) => {
                // sad that match doesn't support if let syntax
                Self::TreeSwapTraverse(
                    curr_mode,
                    TreeTraverseOperation::from_char(key_char).unwrap(),
                )
            }
            // Alt+Windows+P
            // I am fine with this technically being two different things to do but one action
            (
                Mode::TabFocus(TabFocusMode) | Mode::ChoosingFocus { .. },
                'p',
                KeyModifiers {
                    ctrl: false,
                    alt: true,
                    shift: false,
                    caps_lock: false,
                    logo: true,
                    num_lock: _,
                },
            ) => Self::PluckPlace(curr_mode),
            // Alt+Windows+Enter swaps actually focused and focusing
            (
                Mode::ChoosingFocus(choosing_focus_mode),
                '\n',
                KeyModifiers {
                    ctrl: false,
                    alt: true,
                    shift: false,
                    caps_lock: false,
                    logo: true,
                    num_lock: _,
                },
            ) => Self::TreeSwap(choosing_focus_mode),

            // Ctrl+Shift+P opens command palette (see: https://github.com/mathkimchi/singularity/issues/11)
            (
                Mode::TabFocus(TabFocusMode) | Mode::ChoosingFocus { .. },
                'P',
                KeyModifiers {
                    ctrl: true,
                    alt: false,
                    shift: true,
                    caps_lock: false,
                    logo: false,
                    num_lock: _,
                },
            ) => Self::OpenCommandPalette(curr_mode),
            // ESC quits command palette (see: https://github.com/mathkimchi/singularity/issues/11)
            // '\u{1b}' seems to be the char for ESCAPE
            (Mode::CommandPalette(command_palette_mode), '\u{1b}', KeyModifiers::NONE) => {
                Self::QuitCommandPalette(command_palette_mode)
            }

            // Logo+t transposes selected tile's container (hor<=>vertical)
            (
                Mode::TabFocus(tab_focus_mode),
                't',
                KeyModifiers {
                    ctrl: false,
                    alt: false,
                    shift: false,
                    caps_lock: false,
                    logo: true,
                    num_lock: _,
                },
            ) => Self::TransposeTileParent(tab_focus_mode),
            // Logo+s swaps selected tile's siblings
            (
                Mode::TabFocus(tab_focus_mode),
                's',
                KeyModifiers {
                    ctrl: false,
                    alt: false,
                    shift: false,
                    caps_lock: false,
                    logo: true,
                    num_lock: _,
                },
            ) => Self::SwapTileSiblings(tab_focus_mode),
            _ => {
                return Err(curr_mode);
            }
        })
    }

    pub fn from_ui_event(mut curr_mode: Mode, ui_event: UIEvent) -> Self {
        if let UIEvent::KeyPress(key_event, key_mods) = &ui_event {
            if let Some(key_char) = key_event.to_char() {
                match Self::handle_char_key_shortcut_presses(curr_mode, key_char, *key_mods) {
                    Ok(shortcut_action) => return shortcut_action,
                    Err(returned_curr_mode) => {
                        curr_mode = returned_curr_mode;
                    }
                }
            }
        }

        match (curr_mode, ui_event) {
            // Key press isn't any of the keyboard actions; forward it to focused
            (Mode::TabFocus(tab_focus_mode), UIEvent::KeyPress(key, modifiers)) => {
                Self::ForwardKeyPressTab(tab_focus_mode, key, modifiers)
            }
            // Key press isn't any of the keyboard actions; forward it to command palette
            (Mode::CommandPalette(command_palette_mode), UIEvent::KeyPress(key, modifiers)) => {
                Self::ForwardKeyPressCommandPalette(command_palette_mode, key, modifiers)
            }
            // currently, the only None case is when non-shortcut is performed on tab choosing mode
            (Mode::ChoosingFocus { .. }, UIEvent::KeyPress(..)) => Self::NoAction(curr_mode),
            // Resized. Currently ignore.
            (_, UIEvent::WindowResized(_)) => Self::WindowResized(curr_mode),
            // Mouse press
            (_, UIEvent::MousePress(location, container)) => {
                assert_eq!(container, DisplayArea::FULL);
                Self::MousePress(curr_mode, location)
            }
        }
    }
}
```

Now I am going to revert the changes and commit just the devlog.

I got a very basic spawner in command palette to work, but I am not particularly proud of my implementation.
I will mark [#11](https://github.com/mathkimchi/singularity/issues/11) as closed for now,
but there are things in the github discussion that I want to revisit later.

Also, I am probably going to take a few breaks because I have to study for physics and do academic coding projects.

## [#16](https://github.com/mathkimchi/singularity/issues/16)

Did a bit of brainstorming in the issues.

2025/03/13

I will put the plugin api inside `singularity_sap`,
since this is similar.
I am looking at Zellij's [`ZellijPlugin` trait](https://docs.rs/zellij-tile/latest/zellij_tile/trait.ZellijPlugin.html) and Zed's [`extension_api`](https://github.com/zed-industries/zed/tree/main/crates/extension_api) for inspiration.

This is actually kind of similar to my [old implementation of tabs](https://github.com/mathkimchi/singularity/blob/19d6deb7ee7612ab096ded41a324e4e41da6e508/singularity_common/src/tab/mod.rs).

For the wasm in rust itself, these are resources:
- https://benw.is/posts/plugins-with-rust-and-wasi
- https://blog.wasmer.io/executing-webassembly-in-your-rust-application-d5cd32e8ce46

2025/03/14

Okay, the [new thread entry](https://github.com/mathkimchi/singularity/issues/16#issuecomment-2725875195) says all that needs to be said.

I will commit the WASM attempt now and revert everything except for this devlog.

2025/03/24

I have been kind of burnt out, but I want to work on improving the type system for the packets, being the events, requests, queries, and responses.
Since I will add query-response for between plugins, I might add a timeout duration for query.

Brainstorm usage:

```rust
pub struct ShortcutEvent {
    key_char: char,
    command_key: bool,
}

packet_union! {
    name: MyEventUnion,
    // packet: Event,
    packets: [
        ShortcutEvent,
        // python kwargs syntax
        **StandardEventUnion,
        **OtherEventUnion,
    ],
}
```

For additional safety, I might also want to additionally be able to specify: `MyEventUnion: PacketUnion<EventPacketType>`.

2025/03/26

The first thing I want to do is to differentiate the PacketUnion as its own trait.
Previously, I was using the Datable's to_data and try_from_data to convert between packet union object (eg `MyEvents`) and data,
but I will give packet union trait seperate functions to avoid confusion.
Additionally, if possible, I will start by only changing events, and leaving requests alone (query and response were always a bit different).

I'm not sure if I want to make a derive proc macro (current before changes) or declarative macro (tried a long time ago, but didn't like that I could use further macros).

The problem with blind recursive where packetunions themselves just act like packets with their own packet ids is that
because we don't know the depth and what is a fundamental packet vs packet union,
if there is a packet union with a packet union inside of it, then we will match the packet type id to the packet union instead of each of the packets within the inner packet union.
My brain is kinda fried today, does that make sense?

When recieving a packet, use the generic of packet union, when sending a packet, use the packet type.

Lines 1968-2011 of the DEVLOG define the old structure of query and response packets in data form:

```markdown
Query raw data will contain:
- Packet length
  - already handled by the byte writer and reader
  - const size
- Packet type
  - To say that it is a query and not a request
  - (I could eliminate this by just saying everything is a query)
  - For all queries, this should be a constant value
- Query instance id
  - Unique to each instance of a query (eg, even if you query size multiple times, each query will have a different instance id)
  - const size like u64
- Query type id
  - This actually says what type of query the query is
  - This would distinguish between things like: QueryName vs QuerySize
  - const size like u64
- Query inner data (Optional)
  - This would be defined by the query type

It might make more hierarchical sense to put the query type id
before the instance id, but I think practically it makes more
sense to do instance id first, because instance id should be read
even if the query type id is unknown.
(it really doesn't matter, even though my reasoning is kind of bad)
Oh, another reason is that if I did have query bundles,
and stored query type hierarchically in the raw data,
then it would be better to have the instance id first
(even though storing hierarchy would be inefficient).

Response raw data will contain:
- Packet length
  - already handled by the byte writer and reader
  - const size
- Packet type
  - To say that it is a response
  - For all requests, this should be a constant value
- Prompt Query instance id
  - Would be the same as the query instance id that prompted this response
- Response type id
  - This should be easy to tell from the query, so I am considering just not having this
  - Will have a special id reserved for unknown queries
  - If the response has a wrong type id that isn't the null id, then panicing will be understandable
  - const size like u64
- Response inner data (Optional)
  - This would be defined by the response type

For unknown queries, the server should just give a response with
a special `Null` response type id (probably like 0).
I considered having a `NullResponse` packet type, a `Null` response type id,
no response type id and inner data on the null response,
or just having an additional boolean represent
whether the response is null or not.
```

With the new terminology (packet type -> packet category, query/response type id -> packet type id) and new ordering (put query/response type id above prompt query instance id, to standardize this between the different categories), the new description should be:

Query raw data will contain:
- Packet length
  - already handled by the byte writer and reader
  - const size
- Packet category
  - To say that it is a query and not a request (other options, that don't make sense are event and response)
  - (I could eliminate this by just saying everything is a query)
  - For all queries, this should be a constant value
- Query type id
  - This actually says what type of query the query is
  - This would distinguish between things like: QueryName vs QuerySize
  - const size like u64
- Query instance id
  - Unique to each instance of a query (eg, even if you query size multiple times, each query will have a different instance id)
  - const size like u64
- Query inner data (Optional)
  - This would be defined by the query type

Response raw data will contain:
- Packet length
  - already handled by the byte writer and reader
  - const size
- Packet category
  - To say that it is a response
  - For all requests, this should be a constant value
- Response type id
  - This should be easy to tell from the query, so I am considering just not having this
  - Will have a special id reserved for unknown queries
  - If the response has a wrong type id that isn't the null id, then panicing will be understandable
  - const size like u64
- Prompt Query instance id
  - Would be the same as the query instance id that prompted this response
- Response inner data (Optional)
  - This would be defined by the response type

...

I will get started on implementing the `PacketUnion` macro tomorrow.
I think the course of action will be to:
1. expand the `PacketUnion` derive macro in `standard_packets`'s `DisplayEvent` (currently implements `Datable`),
2. manually modify it to let `DisplayEvent` implement `PacketUnion` instead,
3. then use that as a reference to update the general `PacketUnion` macro.

That takes care of packet unions of packets, but not packet unions of other packet unions.

(Some of my friends think I commit too much just to boost my git statistics.
Well I could commit right now, but I'm not, so take that, glolichen.)

Actually, I will commit right now.
Not for git statistics (maybe just a bit because number go up, monke brain go "ooh ooh aah aah"),
but primarily for organization and incremental progress.

2025/03/27

Chorus got let out early (8:30), so I am going to try to squeeze in a commit.

The PacketUnion is actually much simpler than the datable counterpart,
it is just match statements now.

Old expansion (ignore the `const _` boilerplate):

```rs
#[automatically_derived]
impl ToData for DisplayEvent {
    fn to_data(&self) -> Vec<u8> {
        let (id, inner_data) = match self {
            Self::UIEvent(inner_packet) => (
                <UIEvent as PacketTrait>::PACKET_TYPE_ID,
                inner_packet.to_data(),
            ),
            Self::Resize(inner_packet) => (
                <ResizeEvent as PacketTrait>::PACKET_TYPE_ID,
                inner_packet.to_data(),
            ),
            Self::Focused(inner_packet) => (
                <FocusedEvent as PacketTrait>::PACKET_TYPE_ID,
                inner_packet.to_data(),
            ),
            Self::Unfocused(inner_packet) => (
                <UnfocusedEvent as PacketTrait>::PACKET_TYPE_ID,
                inner_packet.to_data(),
            ),
            Self::Close(inner_packet) => (
                <CloseWarningEvent as PacketTrait>::PACKET_TYPE_ID,
                inner_packet.to_data(),
            ),
        };
        let id_bytes: &[u8] = &id.to_be_bytes();
        [id_bytes, &inner_data].concat()
    }
}
#[automatically_derived]
impl TryFromData for DisplayEvent {
    fn try_from_data(data: &[u8]) -> Option<Self> {
        let (id_bytes, inner_data) = data.split_at((PacketId::BITS / 8) as usize);
        let id = PacketId::from_be_bytes(id_bytes.try_into().unwrap());
        match id {
            <UIEvent as PacketTrait>::PACKET_TYPE_ID => {
                Some(Self::UIEvent(UIEvent::try_from_data(inner_data)?))
            }
            <ResizeEvent as PacketTrait>::PACKET_TYPE_ID => {
                Some(Self::Resize(ResizeEvent::try_from_data(inner_data)?))
            }
            <FocusedEvent as PacketTrait>::PACKET_TYPE_ID => {
                Some(Self::Focused(FocusedEvent::try_from_data(inner_data)?))
            }
            <UnfocusedEvent as PacketTrait>::PACKET_TYPE_ID => {
                Some(Self::Unfocused(UnfocusedEvent::try_from_data(inner_data)?))
            }
            <CloseWarningEvent as PacketTrait>::PACKET_TYPE_ID => {
                Some(Self::Close(CloseWarningEvent::try_from_data(inner_data)?))
            }
            _ => None,
        }
    }
}
```

Manual fixed implementation:

```rs
#[automatically_derived]
impl PacketUnion for DisplayEvent {
    fn packet_to_data(&self) -> (PacketTypeId, Vec<u8>) {
        match self {
            Self::UIEvent(inner_packet) => (
                <UIEvent as PacketTrait>::PACKET_TYPE_ID,
                inner_packet.to_data(),
            ),
            Self::Resize(inner_packet) => (
                <ResizeEvent as PacketTrait>::PACKET_TYPE_ID,
                inner_packet.to_data(),
            ),
            Self::Focused(inner_packet) => (
                <FocusedEvent as PacketTrait>::PACKET_TYPE_ID,
                inner_packet.to_data(),
            ),
            Self::Unfocused(inner_packet) => (
                <UnfocusedEvent as PacketTrait>::PACKET_TYPE_ID,
                inner_packet.to_data(),
            ),
            Self::Close(inner_packet) => (
                <CloseWarningEvent as PacketTrait>::PACKET_TYPE_ID,
                inner_packet.to_data(),
            ),
        }
    }

    fn packet_try_from_data(packet_id: PacketTypeId, packet_inner_data: &[u8]) -> Option<Self> {
        match packet_id {
            <UIEvent as PacketTrait>::PACKET_TYPE_ID => {
                Some(Self::UIEvent(UIEvent::try_from_data(packet_inner_data)?))
            }
            <ResizeEvent as PacketTrait>::PACKET_TYPE_ID => {
                Some(Self::Resize(ResizeEvent::try_from_data(packet_inner_data)?))
            }
            <FocusedEvent as PacketTrait>::PACKET_TYPE_ID => Some(Self::Focused(
                FocusedEvent::try_from_data(packet_inner_data)?,
            )),
            <UnfocusedEvent as PacketTrait>::PACKET_TYPE_ID => Some(Self::Unfocused(
                UnfocusedEvent::try_from_data(packet_inner_data)?,
            )),
            <CloseWarningEvent as PacketTrait>::PACKET_TYPE_ID => Some(Self::Close(
                CloseWarningEvent::try_from_data(packet_inner_data)?,
            )),
            _ => None,
        }
    }
}
```

Yooo, I finished in 20 mins, not bad.

...

I'll work on resolving the remaining compile-time errors.

I'll make `EventPacketUnion` and `RequestPacketUnion` macros that also automatically call the `PacketUnion` macro.

YOOO, passed all tests first try.
(That might just mean my tests aren't thorough though)

...

The running also ran normally first try.
I'll make the packet union macro support inner packet unions.
I'll do this by making the user specify the attribute.
In the case the id doesn't match any of the outermost packets,
just see if any of the sub unions return a some.
I wish there was a more compile time-ish way to do this,
where the packet union leaves a special meta thing, but whatever.

Manual implementation:

```rust
// #[derive(EventPacketUnion)]
pub enum Event {
    // #[sub_union]
    DisplayEvent(DisplayEvent),
}
impl PacketUnion for Event {
    fn packet_to_data(&self) -> (PacketTypeId, Vec<u8>) {
        match self {
            Self::DisplayEvent(inner_packet_union) => inner_packet_union.packet_to_data(),
        }
    }
    fn packet_try_from_data(packet_id: PacketTypeId, packet_inner_data: &[u8]) -> Option<Self> {
        match packet_id {
            _ => {
                DisplayEvent::packet_try_from_data(packet_id, packet_inner_data)?;
                None
            }
        }
    }
}
```

...

I just realized, the try operator doesn't work like that.
It almost works the exact opposite of what I want it to do.

```rust
// #[derive(EventPacketUnion)]
pub enum Event {
    // #[sub_union]
    DisplayEvent(DisplayEvent),
}
impl PacketUnion for Event {
    fn packet_to_data(&self) -> (PacketTypeId, Vec<u8>) {
        match self {
            Self::DisplayEvent(inner_packet_union) => inner_packet_union.packet_to_data(),
        }
    }
    fn packet_try_from_data(packet_id: PacketTypeId, packet_inner_data: &[u8]) -> Option<Self> {
        match packet_id {
            _ => {
                if let Some(inner_packet_union) = DisplayEvent::packet_try_from_data(packet_id, packet_inner_data) {
                    return Some(Self::DisplayEvent(inner_packet_union));
                }
                None
            }
        }
    }
}
```

Sweet, worked first try.

...

I guess I got a bit sidetracked with all the packet union macro stuff,
since [#16](https://github.com/mathkimchi/singularity/issues/16) is about plugins.
I should've made a new issue, but I felt kind of guilty about continuously switching between
singularity issues without saving progress recently.

But this kind of was needed, because I want to define a `StandardEvent` in singularity sap's
standard packets while dividing it up into the different event types.

REVIEW: (I was thinking about it, and since I made a new set of functions for PacketUnions,
I might've just made seperate things for packets and unions where the packet itself checks
if the packet id matches. That reduces code repetition, which is good, but it also increases
flexibility which is usually good, but in this case I worry that increased flexibility
will make it easier to make errors.)

I am adding one more macro, the `Query` macro, which is very basic.
While writing it, I realized I misspelled identifier 20 times in just the macro lib.rs file.
I might try running a spellchecker on the whole codebase.
I'm not sure if there will be more typos in my entire source code, or in this md file.

I tried to look for the documentation on matching token streams and how to use attributes,
and sadly, the proc macro world of rust is very sparcely documented.
Making a MathKimchi video on how to do proc macros might not be the worst idea.
I could show off my workflow and tips that I used in singularity.

Okay, I've been working straight from 2:40 to now (4:00) as well as earlier today,
so I'll commit now and not touch singularity until I finish all my homework.

2025/03/30

I want to support communication between singularity applets now (I like this new term `applet` because it encompasses tab, plugin, and subapplication in my mind).
IAC (inter applet communication, like IPC) should come in two types:
- Broadcasts
  - Reciever chooses the sender (or the category of broadcasts to listen to that can be sent by anyone (make sure to ignore broadcasts send by self))
  - Multiple recievers allowed
  - Logical flow: reciever sends a subscribe request, for each broadcast message the sender sends a broadcast request and every reciever gets a broadcast event
  - Sender sends something analogous to the server's `event`
- Direct Communication:
  - Sender chooses reciever (sender can send `query` or `request`)
  - Logical flow:
    - Sender sends `IACQuery` or `IACRequest` to server
    - Server sends `QueryRecievedEvent` or `RequestRecievedEvent` to reciever
    - If a query was sent, then the reciever either sends back a `IACResponse` to the server, which is actually a request. (Technically, it could just not respond and screw everyone over. A cooperative reciever would at least reply with a `QueryUnknownResponse`. I'm actually going to ignore `IAC` queries for now)

In either type, when one applet chooses the other applet, they use the other applet's `Id`.
In practice, the server would offer queries and stuff to help find other applets.
TODO: store applet types as well? (some would have to be anonymous though by nature)

I wish I was careless about resources.
I mean, in contrast to other processes, the overhead of having a bunch of connections
(less than 100 for normal usecases) should be insignificant.
But, for some reason, a primitive instinct is prohibitting me from implementing the "elegant solution".
Let me describe things I would want if resources were not a problem:

- For each query from applet to server, instantly give back a seperate one-time channel for the response so they can choose to wait and listen now or just check back later. Then, disregard like a burner phone.
- A broad listener applet would be like mpsc for each type of thing to listen to
- For each direct communication, start a new channel
- Each applet instance has an applet channel (like now), but for each display it creates, a new display chat is created for all communication regarding that display (eg: DrawRequest, KeyPressEvent, and SizeQuery would be sent via display chat).

The hope would be that those ways are safer and make more logical sense as opposed to using id's to try to pack everything into one stream.

I looked at [this reddit post](https://www.reddit.com/r/rust/comments/7i4ljy/question_mpsc_over_mapped_memory/)
and [ipc-channel](https://github.com/servo/ipc-channel) by servo looks literally perfect for my usecase.
It is literally begging me to use it.
An active repo (last commit just 27 days ago), used by an established organization, almost 1000 stars,
seems perfect for my usecase, but idk.

2025/04/01

Happy April Fools!

I am going to split the project files into `ProjectConfigs` and `Session`.

2025/04/02

I am going to further refactor the project files,
to give uuid's for each known applet type.
...
Actually, it might be better to identify applet types by more consistent/deterministic method,
because there is a level of continuity between the `file_manager` applet in one project and in another.
Currently, I am just using a string like `"file_manager"`.
I could also use a more complicated datatype, (eg: struct to also store version)
(eg2: enum to differentiate different types like UnixSocket connection, piped stdio, etc but this is
actually unnecessary since this should be stored in the applet list,
not as the identifier).
Or, I could use the Id map (which is not consistent) to map applet id to applet data,
but also have a seperate hashmap to map applet name to applet id
(if speed is negligable, more elegant imo to just store name in the applet data and iter to search by name).

Okay, I am actually trying stuff, and I have a few possibilities in mind:
- Generate "id" by hashing string names
  - Just assume collisions won't happen
- Generate "id" as a constant in crates, the same way I did PACKET_TYPE_ID
- Don't use "id" just use names
- Store a mapping of names to ids, and generate id non-deterministically

2025/04/03

I think the optimal way to go is do everything based off the name, so if I do use id's,
I would make it solely generated from the string.
I decided this because I was thinking about url's and git branches.

I'm probably going to use just strings to identify applet, but I might add a wrapper for type-safety.

2025/04/13

After working on this for a few days, I resolved all the compile time errors but it still doesn't work.

2025/04/15

Oh no, I think the actions aren't showing because of numlock, which I turned on with my external keyboard,
but right now my laptop keyboard doesn't have that toggle.
For now, I just erased NumLock from my code, since it is kind of useless anyways.

2025/05/19

I haven't worked on this for a while.
I wrote down [some ideas in the thread](https://github.com/mathkimchi/singularity/issues/16#issuecomment-2888798002).

I think wasm or dynamic libraries's are worth looking into for reactive plugins.

...

Okay, I was looking through my code, and I remember what I was doing now:
I was working on IAC.
I was implementing the `Basic Features`.

2026/05/20

I gave an LLM the last github issue I wrote, and it suggested using dynamic libraries.

I looked up `rust plugins with dynamic libraries` on google,
and accidentally discovered a crate called
[`dynamic_plugin`](https://docs.rs/dynamic-plugin/latest/dynamic_plugin/),
and it seems like what I want to do but it only has 4000 users so I won't use it.
Bevy also has a [dynamic plugin crate](https://crates.io/crates/bevy_dynamic_plugin).
(I think this is different from that other Bevy thing I was looking at; maybe not)
I will dig through both crates' source code to see what they are doing.

dynamic_plugin is actually pretty cool.
The main library only uses a few crates: `libloading`, `thiserror`, `libc`, and `sa` (static assertions).
Actually, the main library is nothing and the actual value seems to come from the macros.
The macros are too big brained for me.
TBH, there is a very good case for just using this crate right now.
Okay, I am going to use `dynamic_plugin` and then `cargo expand` it.

2025/05/29

I pretty much copied the [example host and plugin](https://github.com/lilopkins/dynamic-plugins-rs/blob/main/example-plugin/src/lib.rs) into [dynamic_plugin_sandbox](./dynamic_plugin_sandbox/)
and then expanded the macros, then made it more understandable.

I'll commit what I learned, and the next step would be to set this up for sap.
The template interface would go in sap,
ideally the plugin runner would be in a crate only sde used but in practice I'll probably put it in sap,
and the plugin making macro would ideally be in sde but it might be easier to have it in sap.
Maybe I could add cargo features for sap to have client and server specific code without needing a new crate.

2025/05/30

I will actually more or less deprecate the unix socket and pipes for now
while I work on dynamic library.
Also, I will ignore the waiting stuff for now.

2025/06/03

I have a problem that I can't use closures as extern "C" functions,
which makes sense because the extern "C" function expects a pointer to a function or something;
I can't rigorously explain it but it just makes sense to me.

Let me explain my usecase with an example
(I haven't tested if anything works other than checking for compiler errors, so...).
Suppose I had an app, as well as a library for printing things pretty.
Lets start with something like:

```rust
// lib.rs
#[no_mangle]
extern "C" fn print_statistics() {
    println!("The gravitational constant on Earth is 9.81m/s^2!")
}

// main.rs
pub fn main() {
    let print_statistics: extern "C" fn() = print_statistics; // in practice we'd load the print_statistics function
    print_statistics();
}
```

but now, what if we wanted the pretty print library to be able to print changing values
like a version number?

```rust
// in a shared library
#[repr(C)]
pub struct AppState {
    version_number: u8,
}

// lib.rs
#[no_mangle]
extern "C" fn print_statistics(app_state: &AppState) {
    println!(
        "The version number is {}! The next version will be {}!",
        app_state.version_number,
        app_state.version_number + 1,
    );
}

// main.rs
pub fn main() {
    let app_state = AppState { version_number: 10 };

    let print_statistics: extern "C" fn(&AppState) = print_statistics; // in practice we'd load the print_statistics function
    print_statistics(&app_state);
}
```

Now, what if `print_statistics` needed to print things dependent on a function provided by the app?
IE, what if the dynamic library needs to call a function provided by the caller?

This is still quite simple:

```rust
// lib.rs
#[no_mangle]
extern "C" fn print_statistics(f: extern "C" fn(u8) -> u8) {
    println!("The y-intercept of f is f(0)={}", f(0));
}

// main.rs
extern "C" fn f(x: u8) -> u8 {
    x + 2
}
pub fn main() {
    let print_statistics: extern "C" fn(extern "C" fn(u8) -> u8) = print_statistics; // in practice we'd load the print_statistics function
    print_statistics(f);
}
```

but now, what if the function `f` was a closure (it captures variables from its surrounding scope)?
For example, say f returned the app's version 
With Rust, we could do something like:

```rust
// in a shared library
#[repr(C)]
pub struct AppState {
    version_number: u8,
}

// lib.rs
fn print_statistics(get_next_version: impl Fn() -> u8) {
    println!("The next version will be {}", get_next_version());
}

// main.rs
pub fn main() {
    let app_state = AppState { version_number: 5 };
    let get_next_version = move || app_state.version_number + 1;

    let print_statistics = print_statistics; // in practice we'd load the print_statistics function
    print_statistics(get_next_version);
}
```

The closure `get_next_version` accesses `app_state`.
But we can't do this for dylibs,
because closures in general aren't supported,
and a pointer to such a closure would somehow need to also capture the app state it refers to.

```rust
// in a shared library
#[repr(C)]
pub struct AppState {
    version_number: u8,
}

// lib.rs
#[no_mangle]
extern "C" fn print_statistics(
    app_state: *const AppState,
    get_next_version: extern "C" fn(*const AppState) -> u8,
) {
    println!("The next version will be {}", get_next_version(app_state));
}

// main.rs
extern "C" fn get_next_version(app_state: *const AppState) -> u8 {
    unsafe { (*app_state).version_number }
}

pub fn main() {
    let app_state = AppState { version_number: 10 };

    let print_statistics: extern "C" fn(*const AppState, extern "C" fn(*const AppState) -> u8) =
        print_statistics; // in practice we'd load the print_statistics function
    print_statistics(&app_state, get_next_version);
}
```

In other words, we just make the captured state (aka context)
an argument to a static function.

Note the use of raw pointers (`*const AppState`) instead of references (`&AppState`),
even though both technically compile.
This stack overflow [post](https://stackoverflow.com/questions/71749287/why-do-most-ffi-functions-use-raw-pointers-instead-of-references)
explains better than me.
(In other words, I don't understand why. I just do it bc it is standard.)

2025-06-09

I finished the above blog-ish blob.
I should commit, but I began writing code even before writing the blog thing,
and sunk cost fallacy dictates
that I should finish writing that code before finishing.

2025-06-11

The issue I'd like to address now is letting query return a vec of bytes,
which is harder than it seems.
Passing immutable bytes as an argument is easy:
we can just send a pointer to the start of the slice as well as the length,
and we don't worry about memory safety because
the caller is still in charge of freeing the slice.
But for returning bytes,
there isn't such a simple way.
[This thread](https://users.rust-lang.org/t/how-to-return-byte-array-from-rust-function-to-ffi-c/18136)
mentions some ways.

The big problem is freeing memory.
Rust has actually sheltered me pretty well from directly
thinking/worrying about memory safety,
but I will try to explain memory to the best of my abilities.
When we call an external function,
we expect everything that function is given ownership of and creates
to be freed when that function returns (except for things it returns).
I was going to list more rules, but actually, that is kind of it.
Just the basics of ownership.
(I don't really know where I was going with that.
I was kind of hoping I'd list down the premise then think of a clever solution.)

The safest (imo) is by using an output buffer.
The external function doesn't actually return anything,
it modifies an output buffer that was given to it by the caller as an argument.
But as it stands, the output buffer limits how large the output can be.
So, we could set up an elaborate system with more functions
where the external library somehow tells the caller how long the output is going to be,
then the caller allocates an output buffer of that length,
then the external library writes to the output buffer.
Look at `uncompress` in the
[Rustonomicon FFI page](https://doc.rust-lang.org/nomicon/ffi.html).
I don't like this though.
Maybe I could do a higher layer abstraction where this is done in the background,
but it feels like I am sacrificing performance for no good reason.

The aforementioned rust-lang thread offers the solution that I want to implement.
The external function returns a raw pointer to the bytes and the length.
It is scary though, since it plays with `std::mem::forget`
and creating and dropping it from the raw pointer
(even scarier is that [`forget` is safe because Rust doesn't gurantee no memory leaks](https://stackoverflow.com/questions/74824779/why-is-it-considered-safe-to-memforget-boxes)).
The external code should provide the freeing apparatus.

Another safe way I just thought of is using files like in IPC.
This is overkill though.

...

This is implementation 1:

```rust
/// Whoever owns this object is in charge of freeing the slice this points to.
/// Don't modify this though; I don't know what would happen if you modify this.
///
/// From: https://users.rust-lang.org/t/how-to-return-byte-array-from-rust-function-to-ffi-c/18136/4.
#[repr(C)]
pub struct OwnedCBytes {
    bytes_ptr: *mut u8,
    len: usize,
    /// REVIEW: check if this is actually needed; the rustlang thread doesn't use it.
    capacity: usize,
    free: extern "C" fn(&mut Self),
}
impl From<Vec<u8>> for OwnedCBytes {
    fn from(mut value: Vec<u8>) -> Self {
        let bytes_ptr = value.as_mut_ptr();
        let len = value.len();
        let capacity = value.capacity();

        // https://stackoverflow.com/questions/74824779/why-is-it-considered-safe-to-memforget-boxes
        // this memory is leaked here but will be freed in the `free_vec` function, which is called exactly once in `drop`
        std::mem::forget(value);

        /// https://users.rust-lang.org/t/how-to-return-byte-array-from-rust-function-to-ffi-c/18136/13?u=mathkimchi
        extern "C" fn free_vec(bytes: &mut OwnedCBytes) {
            let vec = unsafe { Vec::from_raw_parts(bytes.bytes_ptr, bytes.len, bytes.capacity) };
            // no need to manually call drop, but I just wanted to highlight it
            drop(vec);
        }

        Self {
            bytes_ptr,
            len,
            capacity,
            free: free_vec,
        }
    }
}
impl Drop for OwnedCBytes {
    fn drop(&mut self) {
        // free the forgotten vec
        (self.free)(self);
        // the rest will be freed normally
    }
}
```

I liked it as an improvement of the rust thread solution,
because it was super abstract and stuff,
but it gives me the icks just a little.
I'll commit this current implementation though.

The capacity is really annoying with this.
It just feels so arbitrary.
I could further abstract into:

```rust
// worst OOP ever, lol
#[repr(C)]
pub struct OwnedCBytes {
    ptr: *mut c_void,
    get_len: extern "C" fn(*const c_void) -> usize,
    /// fills the buffer, which is expected to be length of `get_len`
    get_bytes: extern "C" fn(*const c_void, *mut u8),
    free: extern "C" fn(*mut c_void),
}
```

(`c_void` represents an opaque type)
but I don't want to do this.
This is just OOP but horrible.
(Actually, this might be useful later.)

Instead, I am going to do the opposite approach and specify for rust Vecs.
I suppose I am just going all-in on the assumption that both sides are written in rust
and will use the shared libraries provided by me.
(Given that I am likely the only person who will use and even-more-so develop for singularity,
I'd say that is a fair assumption.)

---

SIDE NOTE: I noticed I could do `ManuallyDrop::new(value).capacity`
but manual drop is defined as:

```rust
pub struct ManuallyDrop<T: ?Sized> {
    value: T,
}
```

so you'd assume the proper syntax is `ManuallyDrop::new(value).value.capacity`.
Apparently it is from the `Deref` trait.
TODO: harness this for `(pub` matches or grep for certain structs.

---

I asked ChatGPT about the memory safety of:

```rust
impl Drop for CVec {
    fn drop(&mut self) {
        // REVIEW: is this going to double free?
        let vec: Vec<u8> = unsafe { Vec::from_raw_parts(self.bytes_ptr, self.len, self.capacity) };
        // unnecessary but highlights that the vec is dropped
        std::mem::drop(vec);
    }
}
impl From<CVec> for Vec<u8> {
    fn from(value: CVec) -> Self {
        let vec = unsafe { Vec::from_raw_parts(value.bytes_ptr, value.len, value.capacity) };

        // prevent double freeing the vec in CVec's drop
        std::mem::forget(value);

        vec
    }
}
```

and now I realize I don't understand how memory works in rust.
I am going to watch a video on it.

---

I watched [Visualizing memory layout of Rust's data types](https://www.youtube.com/watch?v=7_o-YRxf_cc)
and it is pretty informative.
It doesn't go over what happens with Forget and Drop,
but whatever.

After the research,
I still am not sure if the code is safe,
I guess I will find out if it bites me in the behind.

2025-06-14

I am now implementing the applet context,
which is like the new ServerHandler.
In doing this, I realized that I should actually deviate from the `UniversalStream`
implementations.

So, I want to explain the protocols/implementations that exist for singularity:
- Datable Trait
  - `Datable`, `ToData`, `TryFromData`
  - Just a rust trait for converting an object to bytes and vice versa, where the object's type is known.
  - This allows for sending objects with pre-known types through FFI and sockets and saving objects to files.
- Packet Trait System
  - `PacketTrait` is a `Datable` with a type id.
  - An implementor type of `PacketUnion` represents a bundle of different types that implement `PacketTrait`.
  - An object instance `PacketUnion` represents a specific `PacketTrait` object.
  - Given a `PacketTrait`'s byte representation + the `TypeId` of the `PacketTrait` the bytes came from, if that `PacketTrait` is in a `PacketUnion` type's bundle, the `PacketUnion` type will create an object instance of itself.
  - Allows for sending and storing objects with multiple possible types (the multiple possible `PacketTrait` types are bundled into `PacketUnion`).
- Categorized Packet Traits
  - Just `PacketTrait`'s and `PacketUnion`'s with category (Event, Request, Query, Response) specified for additional safety.
  - `UniversalQueryTrait`, `EventPacketTrait`, `RequestPacketTrait`, `EventPacketUnion`, `RequestPacketUnion`
- Byte Stream Trait
  - `ByteStream`, `ByteReader`, `ByteWriter`
  - Deals in chunks of bytes, where the length matters and isn't constant. ('Hi' is different from 'H' then 'i')
  - Flexible reading:
    - Poll (gives all byte chunks currently readable)
    - Waits (waits until a byte chunk is readable and then returns it)
  - Writing:
    - Send a byte chunk
  - Implementation Note: many implementations send the length first and then sends the actual byte chunk
- Universal Stream Structs
  - `UniversalClientStream`, `UniversalServerStream`
  - Given a byte stream object and with recieving bundle types (eg: a `EventPacketUnion` type for the client's universal stream), the universal stream is able to send `PacketTrait` or `PacketUnion` objects and recieve `PacketUnion` objects
  - Is implemented (everything above is a protocol with open implementation).
  - The byte-level structure of the packets that I described earlier (like in 2025/03/26) are pretty much just for the Universal Stream implementation
- Dylib Applet
  - I am working on this right now. The scope of it is currently unknown.

With active applets, the SDE and applet talk to each other via universal stream,
where they use matching byte stream methods.
With reactive applets (`dylib_applet`),
I am planning on making the `dylib_applet` protocols.

In both methods, they send categorized packet traits to each other.

I am compelled to also provide specific methods to dylib applet,
for reasons like performance.

...

Dylib Applet feels so out of place in `singularity_sap`,
I felt like I should put it in its own crate.
But, I realized that rust modules exist for a reason;
I should stop making new crates and just make modules with cargo cfg features.

Side note: Wowie, 10k lines of singularity? Noice!

...

I will add a `ratk`,
which will be the `reactive applet toolkit`

2025-06-16 1:17AM

I finished writing the `register_applet` macro.
I haven't tested it yet (that is the next step, to make a dylib client).
The macro itself is very simple, and the bulk of the time I spent on
developing today was avoiding code repetition by implementing packet to and fro `typed_data`,
as well as figuring out lifetimes for `CBytes`.
I also tried to clean up some code.

I don't know why I am trying to write down excuses for why I worked unreasonably
long on this commit.
If I had to guess, it is probably because I am trying to cope with the fact
that I tried to multitask watching Last Week Tonight while working on singularity
for an hour or two (or three possibly, sue me) and realized that
I should only code until I finish this commit,
and then I can focus entirely on whatever else I want.
Welp, now it is 1:25AM, and I am starting a lab tommorow
(technically today, 2025-06-16).
I want to wake up at 7AM, eat breakfast, and get to the lab early, like 8:30AM,
even though for the last 3 weeks of summer, I've been sleeping after 2AM and waking up at 12-3PM.

I wanted to finish a course of 16, 1 hour lectures before starting the lab,
but I think I watched the important parts, so that won't worry me to the point of not sleeping
(instead, that will simply be my inconsistent sleeping habit).
Anyways, I guess am kind of stalling now because I need to shower,
and I kind of don't want to, even though once I start showering I enjoy the warmness of it.

It is weird that I hijacked this devlog entry with personal info.
I already write relatively a lot about my personal life in my devlogs,
but this is more than usual.
I am not sorry for adding this entry to the singularity devlog,
but I am feeling dejected
(it's the feeling of to sighing and saying whatever then reluctantly following along with what is happening,
but it's weird since I am the one who is making it happen),
because I have a seperate journal specifically for personal thoughts,
as well as a seperate diary for events in my personal life,
and I am currently messing up the organization of those journals.

But, once I get the documentation/devlog/journal/diary writing applet working,
these organizational problems will be of the past.
(HA, NOICE!
I was able to make this entry somewhat relevant to singularity,
thereby justifying its location!
Truly, a genius maneuver.)

... well, now this entry does belong here,
so me talking about how it ruins my organization should be removed,
but if that is removed, then the entry wouldn't be relevant to singularity,
so I could add back the ramblings,
which would make this entry relevant again.
But then, I'd need to...

Whatever, I'm "going to sleep" now.

2025-06-16

To test dylib applet, I will make a math game.

I wanted to do a sudoku or maze game,
but the logic for both of those is unnecessarily complicated
for a simple demo.

Also, I noticed that all my crates are still 2021,
so I will update them to 2024, which came out a few months ago.

2025-06-19

I am going to write two sets of functions in the `ReactiveApplet` trait
for global events vs instance events.
I also might want a seperate function for creating an applet.
I also might want two seperate events for global vs instance.

2025-08-09 10:16PM

WOW, it has been a while.
Over summer, I have been working on an internship, college apps,
and for my personal project I've been working on celldom.

I have been thinking about singularity, but evidently, I haven't been coding for it much.

I remember I was doing something with dylibs, but I don't know what this commit was.

I don't want to use VSCode, but neovim and emacs are annoying to configure with Nix,
but this is good because it will incentivize me to work on singularity even faster.

I guess I should start by looking at what I've modified and maybe some TODO, REVIEW, and FIXMEs.
Maybe I should actually start by looking at the logs.

Okay, so it seems like my previous commit was me writing a play.
Then, on 2062-06-16, I said I would make a math game to *test* the dylib in this commit,
so it seems I already wrote code for the dylib infastructure.
Then, I updated my crates to rust 2024.
On 2025-06-19, it seems I was going to make two different function typesfor global vs instance events,
and I am not sure if that is still a TODO.
Knowing me, it is probably still a TODO.
On 2025-08-09 10:16PM, it seems like I started logging the time of day in the devlog so I could brag about having no life.
I commented on how I haven't been working on singularity and gave some half-baked excuses.
Then, I babbled a bit about random stuff, and then I started to look at the logs and summarizing them.
Then, I finished singularity, did a backflip, and made it in time for my daughter's ballet recital.
Oh wait, the previous sentence is all halucinated, the most recent thing I did is look at the logs and summarized them, before I took (or, am taking) this chance to become meta.

2025-08-16 8:16PM

Uh, after writing my last entry, I didn't actually do anything.

First, I have to deal with deallocating an Applet instance from memory.
I am just going to add a function called `close_applet` that takes ownership of the Applet object
and therefore is responsible with deallocating it.

2025-08-24 6:03PM

I have to write code to have the sde actually use dylib now.

...

Maybe its because I just lost PeddieHacks or because I am coming back to singularity after so long
or maybe I am just seeing this from the correct perspective for the first time,
but my code is too messy.
I might do a soft restart/refactor by first figuring out what modules are solid,
where the solid modules belong,
and then rethinking the architecture of everything that remains.
I should be able to explain what every single thing's purpose is
by only looking at the name and what module its in,
without needing to read documentation or code.

Either way, I am just going to commit this current mess right now.

...

First, I think that the "util"s are most solid and clearly defined.
Most of `singularity_sap`, `singularity_common` (which only consists of utils rn),
and `singularity_ui` (there is a lot of improvements to be done, but at least it is pretty clear what it is supposed to be) are fine for now.

I want to redo the rest and `singularity_sap::dylib_applet`.

2025-09-20 12:02AM

I have been doing a lot of thinking, and I wrote some stuff down on my paper journal
as well as brainstorming UI on an iPad.

To summarize the big points:
- My progress on singularity has been abysmal because I keep adding "widening" it without following through on a single Minimal Usable Product (I prefer this to MVP because my current sights are set foremostly on getting a usable product ASAP). I have been wandering aimlessly in my codebase adding things that are easy to implement (or at least seem easy to implement).
- I have begun to "manifest" singularity, where I try to imagine using singularity as intensely and vividly as possible. With this, I aim to get the "what" of sinularity solidified such that my worries on the "how" can be directed towards forwards progress. This contrasts to the past where my lack of vision or the "what" has made me implement the "how" in useless directions.
- Steps I will take now:
  - I will commit this and close this branch and issue.
  - Plan new crate structure
  - Purge all subpar code
    - (Even the good ones that I don't absolutely need right now can be purged now and added back later)
  - Most basic applet
    - Only predetermined events
    - Plugins are static-time Rust plugins

I do already have many ideas from my manifestation sessions, but in lieu of my limited time,
I will leave those on my iPad until I begin coding them.

I wanted to call this new Singularity the Nova Singularity,
but I realized that there isn't really an old Singularity to compare it to,
since I am still working on the first MVP.
I guess I can call this the reignition stage of the development era.

2025-09-20 11:24AM

The crates I need are:
- UI
  - Current one is good
- SDE
- STTK
- Some tabs (standard tabs)
- Sporg (but this is kind of a mess as well, might need to start it from scratch.)
- and the common just for help

2025-10-09 2:20AM

I will delete unnecessary crates.

2025-10-11 9:12PM

I'm going to change the code now to get rid of unnecessary files and maybe add some skeleton.
I want to get rid of errors from referencing code that doesn't exist.

Things like the packets, I will put in singularity_common.

Packets (both the abstract protocol and the standard packets) used to be in SAP.
I am getting rid of abstract packets while Singularity is still in pre-alpha.

I think I will do reactive applets as the only way for now.
I won't support processes or multithreading either in pre-alpha.

2025-10-12 9:27PM

I will start out by planning the Handlers,
which represent communication between the SDE and applet.
The `ServerHandler` is used by the Applet and implemented by the SDE.

After thinking about it, I don't need a Ratk as a seperate thing,
because I can just have an `AppletHandler` in singularity_common
(and I'll call it `ReactiveHandler` since that is more clear).

I guess if progress wasn't a priority, I'd put both of these things in singularity SAP,
but it shall not exist while I continue forward.

Actually, I am just going to make sap a folder in singularity_common,
since I was going to make a folder for all the handler and related things anyways.

My vague idea for communication:

- Applet -> SDE:
  - Requests (make queries=requests later?)
  - Queries ask for a response in return
  - Creating other Applets:
    - Make the applet instance on its own and then just give it to the SDE and tell it to register it. (a type of Request)
    - Could also have a more generic/abstract way, but idk how yet. Would still be a request.
- SDE -> Applet:
  - Event: inform applet of event that happened
  - Response to queries
  - Initialization: different from a normal event (currently all events are instance events, no global) because this is called globally and also requires return value

I guess the new thing is the initialization.

2025-11-11 11:26AM

Lowkey, I am super lost because its been months since I actually worked on singularity.

2025-11-12 5:04PM

I explained to the other guy in Micro47 the overview of Singularity.
The main issue is just connecting Applets to the Singularity server.
The four levels of this I've considered (top is most general and difficult to rigid but easy):

1. Processes
   1. Dynamic
2. Dylibs
   1. Dynamic
   2. Reactive
3. Threads
4. Reactive Rust Objects
   1. Reactive
   2. Server Context Object:
      1. The server talks to applet by calling `applet.some_method(..., server_ctx)`
      2. The applet talks back to server by calling `server_ctx.some_method()`
      3. The server passing context as a bundle of callbacks
      4. Metaphorically, when the server calls an applet's method, the server is sending a letter/order to the applet. Including the context is like signing the letter "You can call me for more information or help at _"

I think I had something not bad for threads.
Then, I was like "I want a challenge" and did dylibs.
Then, I got annoyed by how slow I was implementing dylibs and switched to Reactive Rust,
but gave up after that.

So I tried to take 1 step forward, took 2 steps back,
then just laid down.

The really cool thing about Reactive Rust Objects is that maybe applets could have subapplets,
but I don't want to think about that right now.

I am going to try to run through a very basic flow
(simplification of projects for now. Also, tree nodes store views not tabs):
- User starts singularity for the first time
- Singularity creates a tree with just the root which is a view of a hub applet
  - (Hub is like a mix between a terminal and the vscode command palette)
  - Creating a hub applet:
    - Server runs `Hub::new()` and should get `Some`
- User types in: `replace_self(apps.find("EDITOR"))` (syntax is just placeholder)
  - Server relays user event to applet like `applet.handle_event(user_event, server_ctx)`
  - Applet gets the typing information and stores it in its current string or st
  - Applet updates its ui display with `server_ctx.request(...)`
- When the server tells the applet the user's `<ENTER>`, the hub applet spawns an editor child applet
  - Applet runs `let editor = Editor::new().unwrap();` (returns a blank editor without a file)
  - Applet calls `server_ctx.request(Requests::ReplaceSelf(editor))`
- Server replaces hub with applet in the view
- User types stuff
  - Already know how to deal with this
- User closes the editor
  - Server calls `applet.destruct(server_ctx)` (which takes ownership)
  - In production, there should be a way to ask confirmation on non-forced closes, but ignore that for now. Pretend the app just saves the text to instance storage
  - Applet calls `server_ctx.request(...)` to update instance storage
  - Server updates applet's instance storage
  - Applet ends everything it needs to (shouldn't be much)
  - Server removes everything else related to applet (or puts it in the history)

2025-12-03 11:46PM

I don't have anything technical to write right now.

But, I wanted to write down that maybe I should grab myself a whiteboard,
a good hour or two, and maybe a friend/rubber ducky and re-draft all the
types I need as well as the interactions between them (in the form of typed functions).

Oh, I guess I can also say something I thought about the actual tree-hierarchy system of subapps.
An idea I had is that I could do a recursive style implementation of the tree hierarchy.
What I mean is, that SDE (or some part of its framework) could technically
just be a single-applet runner.
But, in practice, most applets can implement `NodeApplet` which would
allow the implementer to implement the Applet itself,
but the Applet itself is enclosed and the NodeApplet has code that allows for it to be a parent
node as well.

This idea is the culmination of a series of smaller change ideas:
1. Just have a 1-to-1 correspondence between a tree node, applet instance, and a view.
2. But what if I want two applets in one view? Create a TilerApplet that is a parent of those two. The TilerApplet's implementation is that it's view is updated by having one half be one child's view and the other side be the other child's view.
3. But this sounds super wasteful if done through the same framework as all the other applets and I want to avoid adding a special exception just for a TilerApplet. Well, what if we made every applet recursive by nature?

Making the entire tree-hierarchy of the applets come from recursion is the crux of the idea.
Most Applets will run inside a NodeApplet,
where the NodeApplet library handles all the tree-related stuff
like setting focus & view and forwarding events/views between SDE (via parent) to
the focused tab (either self or the focused child).

This *is* kinda scary because if one parent messes up the hierarchical duties,
then all its children will also stop working.
But, I think this is fine if I make NodeApplets the default
and people have to go out of their ways to do something else like a SplitView Applet.

Right now, I am leaning towards SDE is just an AppletRunner so I might call it SAR
or something,
and the tree selection stuff is implemented by maybe the RootNodeApplet
which is like NodeApplet but with extra root-related duties
like the tree traversal/selection.

2025-12-04 11:11PM

I am going to just ignore tab restoration for now.

I will have a Singularity App Runner, SAR, instead of SDE.
I will just start by having the SAR and a basic text editor.
SAR will be like Singularity UI where it could theoretically be a general
crate for non-singularity app development.
IE it will be a general app framework
and all the singularity specific stuff (like hierarchy)
will be implemented in the applet toolkits.

2025-12-05 5:34PM

As I am working on the skeleton code, I am not sure how to do the UI.

The obvious plan I had was to just let them update with Queries and whatnot
since I am just doing reactive applets.

In theory, this should be fine but my gut,
motivated perhaps by long-term planning or perhaps greed and ambition,
is telling me I can and should do better right now.
The problem with the reactive approach is that is has absolutely no way to work for active applets.

The best alternative I've thought of right now is by thinking of the applet and SAR relationship
through a new perspective:
instead of SAR holding/owning the applets,
the are two independent things (they don't own each other)
and each have ways to call each other's functions.
If I can make this work within Rust,
then it will be efficient for reactive applets while also being fexible.

2025-12-06 9:10AM

The first thought I had was to use MPSC, but I want it to be even more flexible than that.
Maybe just functions where MPSC is a default implementation.

I don't know how the ordering would work for this.
The naive way is to first create one (of the SAR or the applet) with an Option for communication
and then make the other object and the communication and give it to the original thing.

But I don't like the Option because we know that once everything is initiated,
it will be a Some but we will need to keep calling unwrap.

Another order would be to start by creating the connection and then create the two objects (like MPSC).
But if I do that, then it wouldn't be very flexible.

Or, I could do something slightly similar to the Option idea but just start them out with trivial methods
and allow it to be modified.
I think the term for this is hooks.

So I think I could just have it so that the order is:
1. Create an Applet which has methods: `set_hooks` and `handle_event` and `get_display`, ...
2. Create the SAR and give it the applet
   1. SAR makes everything it needs to
   2. SAR makes hook that implements things like `notify_update_display` from itself
   3. The SAR calls `applet.set_hooks` and gives it the hook

This is actually similar to the reactive applet where I send the server_ctx
as an argument every time the SDE called an applet's funciton,
like `handle_event(event, server_ctx)`.

I think I can assure myself that creating Applet first makes sense
because an Applet should be allowed to exist without a runner,
but a runner needs an applet to run.

Hmmm...
I am not sure how the ownership would work with this
because Hook holds a reference to SDE
but I don't want to deal with lifetimes and all that mess.
I guess MPSC "dealt" with this problem by letting the user not worry about lifetimes
and with no gurantee of the reciever's existance,
but then you get an error if the reciever doesn't exist.

In the current (above) idea with the hooks,
I simply replaced an Optional hook by allowing a trivial implementation instead of a None.
But, here is another perspective:
we can kind of "drag out" the two states of the hook to say
that the Applet itself has two states: an applet with a hook and an applet without a hook.
The `set_hooks` function essentially consumes an applet without a hook
and returns an applet with a hook in its place.
So, what if we just make the before vs after two different functions entirely?
This idea is the same as just having an applet initializer function.
So, the steps taken would be:

1. Create Applet initializer data
2. Create Runner and give it applet initializer data
   1. Runner makes its own things that it needs to
   2. Runner makes the hook
   3. Runner initializes the Applet, giving it the hook

I actually vaguely remember doing this exact thing in the past.
