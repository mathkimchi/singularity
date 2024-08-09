# singularity

## What is singularity?

Singularity is an app that runs and manages sub apps called blocks.
It should roughly do the same things as emacs, just in a different way.
Though singularity should have modular blocks and ui that anyone can write, there are simple default blocks and a ui.

Blocks are managed in a tree-based heirarchy, because I believe that is a convinient way to organize most things (like file directories).

---

TODO: get rid of this later

## Brainstorm

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
    - Todo manager
    - Live viewer (for markup languages)
- Note Taking
  - Need to quickly write and organize and read notes
  - Idk, look into org mode and org roam
- Calendar and Todo
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
- 

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
- Nodes in the tree are either empty (organizational groupers) or an individual window
  - I'm going to have the organization similar to the tree tab extensions
- Subapps to do:
  - [ ] Editor
  - [ ] File Manager
  - [ ] Terminal
  - [ ] Todo manager
